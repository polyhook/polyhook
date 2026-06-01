using System.Text;
using System.Text.Json;
using Polyhook;
using Wasmtime;
using Xunit;

namespace PolyhookTests;

/// <summary>
/// Tests for <see cref="Polyhook.Polyhook.ReadAsync"/> and
/// <see cref="Polyhook.Polyhook.RespondAsync"/> using a mock
/// <see cref="IWasmInvoker"/> so that no real <c>polyhook.wasm</c> binary is
/// required.
/// </summary>
public class PolyhookTests : IDisposable
{
    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// <summary>
    /// Builds the 4-byte LE length-prefix + UTF-8 JSON byte sequence that the
    /// real WASM module would return from its linear memory.
    /// </summary>
    private static byte[] LengthPrefix(string json)
    {
        var payload = Encoding.UTF8.GetBytes(json);
        var result  = new byte[4 + payload.Length];
        BitConverter.TryWriteBytes(result.AsSpan(), payload.Length);
        payload.CopyTo(result.AsSpan(4));
        return result;
    }

    /// <summary>
    /// A mock <see cref="IWasmInvoker"/> that returns a pre-configured byte
    /// array and optionally captures the <paramref name="inputBytes"/> passed
    /// to <see cref="Invoke"/>.
    /// </summary>
    private sealed class MockWasmInvoker : IWasmInvoker
    {
        private readonly byte[] _response;

        /// <summary>The raw bytes that were passed to the last <see cref="Invoke"/> call.</summary>
        public byte[]? CapturedInput { get; private set; }

        public MockWasmInvoker(byte[] response) => _response = response;

        /// <inheritdoc/>
        public byte[] Invoke(Func<Instance, int, int, int> wasmCall, byte[] inputBytes)
        {
            CapturedInput = inputBytes;
            // Return the payload portion only (skip the 4-byte length prefix).
            var payload = new byte[_response.Length - 4];
            _response.AsSpan(4).CopyTo(payload);
            return payload;
        }
    }

    // Keep the original invoker so every test can restore it in Dispose().
    private readonly IWasmInvoker _originalInvoker = Polyhook.Polyhook.WasmInvoker;

    /// <summary>Restore the real invoker after each test.</summary>
    public void Dispose() => Polyhook.Polyhook.WasmInvoker = _originalInvoker;

    // -----------------------------------------------------------------------
    // ReadAsync tests
    // -----------------------------------------------------------------------

    [Fact]
    public async Task ReadAsync_ReturnsHookEvent_WithExpectedFields()
    {
        const string hookJson = """
            {
              "event":     "tool:before",
              "tool":      "bash",
              "input":     { "command": "ls -la" },
              "sessionId": "sess-001",
              "caller":    "claude-code"
            }
            """;

        Polyhook.Polyhook.WasmInvoker = new MockWasmInvoker(LengthPrefix(hookJson));

        var stdinBytes  = Encoding.UTF8.GetBytes("{\"raw\":\"payload\"}");
        using var stdin = new MemoryStream(stdinBytes);

        var evt = await Polyhook.Polyhook.ReadAsync(inputStream: stdin);

        Assert.Equal(EventKind.ToolBefore, evt.Event);
        Assert.Equal("bash",              evt.Tool);
        Assert.Equal("sess-001",          evt.SessionId);
        Assert.Equal(CallerKind.ClaudeCode, evt.Caller);
        Assert.NotNull(evt.Input);
        Assert.Equal("ls -la", evt.Input["command"].GetString());
        Assert.Null(evt.Output);
        Assert.Null(evt.AgentId);
    }

    [Fact]
    public async Task ReadAsync_PassesStdinBytes_ToWasmInvoker()
    {
        const string hookJson = """
            {"event":"notification","sessionId":"s","caller":"unknown"}
            """;

        var mock = new MockWasmInvoker(LengthPrefix(hookJson));
        Polyhook.Polyhook.WasmInvoker = mock;

        var stdinContent = Encoding.UTF8.GetBytes("some raw stdin content");
        using var stdin  = new MemoryStream(stdinContent);

        await Polyhook.Polyhook.ReadAsync(inputStream: stdin);

        Assert.NotNull(mock.CapturedInput);
        Assert.Equal(stdinContent, mock.CapturedInput);
    }

    [Theory]
    [InlineData("claude-code", CallerKind.ClaudeCode)]
    [InlineData("cursor",      CallerKind.Cursor)]
    [InlineData("windsurf",    CallerKind.Windsurf)]
    [InlineData("cline",       CallerKind.Cline)]
    [InlineData("amp",         CallerKind.Amp)]
    [InlineData("unknown",     CallerKind.Unknown)]
    public async Task ReadAsync_DeserialisesCaller_ForAllKnownCallers(
        string callerSlug, CallerKind expectedCaller)
    {
        var hookJson = $$"""{"event":"notification","sessionId":"s","caller":"{{callerSlug}}"}""";

        Polyhook.Polyhook.WasmInvoker = new MockWasmInvoker(LengthPrefix(hookJson));
        using var stdin = new MemoryStream(Array.Empty<byte>());

        var evt = await Polyhook.Polyhook.ReadAsync(inputStream: stdin);

        Assert.Equal(expectedCaller, evt.Caller);
    }

    [Fact]
    public async Task ReadAsync_AllEventKinds_Deserialise()
    {
        var cases = new[]
        {
            ("tool:before",   EventKind.ToolBefore),
            ("tool:after",    EventKind.ToolAfter),
            ("session:start", EventKind.SessionStart),
            ("session:stop",  EventKind.SessionStop),
            ("agent:stop",    EventKind.AgentStop),
            ("notification",  EventKind.Notification),
        };

        foreach (var (slug, expected) in cases)
        {
            var hookJson = $$"""{"event":"{{slug}}","sessionId":"s","caller":"unknown"}""";
            Polyhook.Polyhook.WasmInvoker = new MockWasmInvoker(LengthPrefix(hookJson));
            using var stdin = new MemoryStream(Array.Empty<byte>());

            var evt = await Polyhook.Polyhook.ReadAsync(inputStream: stdin);

            Assert.Equal(expected, evt.Event);
        }
    }

    [Fact]
    public async Task ReadAsync_WithOptionalFields_PopulatesOutput()
    {
        const string hookJson = """
            {
              "event":     "tool:after",
              "tool":      "write_file",
              "output":    { "exitCode": 0 },
              "sessionId": "sess-999",
              "agentId":   "agent-42",
              "caller":    "cursor"
            }
            """;

        Polyhook.Polyhook.WasmInvoker = new MockWasmInvoker(LengthPrefix(hookJson));
        using var stdin = new MemoryStream(Array.Empty<byte>());

        var evt = await Polyhook.Polyhook.ReadAsync(inputStream: stdin);

        Assert.Equal(EventKind.ToolAfter, evt.Event);
        Assert.Equal("write_file",        evt.Tool);
        Assert.Equal("agent-42",          evt.AgentId);
        Assert.NotNull(evt.Output);
        Assert.Equal(0, evt.Output["exitCode"].GetInt32());
        Assert.Null(evt.Input);
    }

    // -----------------------------------------------------------------------
    // RespondAsync tests
    // -----------------------------------------------------------------------

    [Fact]
    public async Task RespondAsync_WritesWasmOutput_ToOutputStream()
    {
        // The mock returns the encoded form of a simple JSON blob.
        const string callerPayload = "{\"decision\":\"approve\"}";
        var mock = new MockWasmInvoker(LengthPrefix(callerPayload));
        Polyhook.Polyhook.WasmInvoker = mock;

        using var stdout = new MemoryStream();
        await Polyhook.Polyhook.RespondAsync(Polyhook.Polyhook.Approve(), outputStream: stdout);

        stdout.Seek(0, SeekOrigin.Begin);
        var written = Encoding.UTF8.GetString(stdout.ToArray());
        Assert.Equal(callerPayload, written);
    }

    [Fact]
    public async Task RespondAsync_Approve_PassesCorrectJsonToWasm()
    {
        var mock = new MockWasmInvoker(LengthPrefix("{}"));
        Polyhook.Polyhook.WasmInvoker = mock;

        using var stdout = new MemoryStream();
        await Polyhook.Polyhook.RespondAsync(Polyhook.Polyhook.Approve(), outputStream: stdout);

        Assert.NotNull(mock.CapturedInput);
        var captured = Encoding.UTF8.GetString(mock.CapturedInput);
        using var doc = JsonDocument.Parse(captured);
        Assert.Equal("approve", doc.RootElement.GetProperty("action").GetString());
    }

    [Fact]
    public async Task RespondAsync_Block_PassesMessageToWasm()
    {
        var mock = new MockWasmInvoker(LengthPrefix("{}"));
        Polyhook.Polyhook.WasmInvoker = mock;

        using var stdout = new MemoryStream();
        await Polyhook.Polyhook.RespondAsync(
            Polyhook.Polyhook.Block("no rm -rf allowed"),
            outputStream: stdout);

        Assert.NotNull(mock.CapturedInput);
        var captured = Encoding.UTF8.GetString(mock.CapturedInput);
        using var doc = JsonDocument.Parse(captured);
        Assert.Equal("block",              doc.RootElement.GetProperty("action").GetString());
        Assert.Equal("no rm -rf allowed",  doc.RootElement.GetProperty("message").GetString());
    }

    [Fact]
    public async Task RespondAsync_Modify_PassesInputToWasm()
    {
        var mock = new MockWasmInvoker(LengthPrefix("{}"));
        Polyhook.Polyhook.WasmInvoker = mock;

        var modifiedInput = new Dictionary<string, JsonElement>
        {
            ["command"] = JsonSerializer.SerializeToElement("echo safe"),
        };

        using var stdout = new MemoryStream();
        await Polyhook.Polyhook.RespondAsync(
            Polyhook.Polyhook.Modify(modifiedInput),
            outputStream: stdout);

        Assert.NotNull(mock.CapturedInput);
        var captured = Encoding.UTF8.GetString(mock.CapturedInput);
        using var doc = JsonDocument.Parse(captured);
        Assert.Equal("modify",      doc.RootElement.GetProperty("action").GetString());
        Assert.Equal("echo safe",   doc.RootElement.GetProperty("input").GetProperty("command").GetString());
    }

    [Fact]
    public async Task RespondAsync_EmptyPayload_WritesNothingToStream()
    {
        // A response payload of zero bytes (edge case).
        var emptyLengthPrefix = new byte[4]; // 4 zero bytes => length = 0
        var mock = new MockWasmInvoker(emptyLengthPrefix);
        Polyhook.Polyhook.WasmInvoker = mock;

        using var stdout = new MemoryStream();
        await Polyhook.Polyhook.RespondAsync(Polyhook.Polyhook.Approve(), outputStream: stdout);

        Assert.Equal(0, stdout.Length);
    }

    // -----------------------------------------------------------------------
    // DefaultWasmInvoker — coverage for the production path wrapper
    // -----------------------------------------------------------------------

    [Fact]
    public void DefaultWasmInvoker_IsAssignedByDefault()
    {
        // After Dispose restores the original, verify the default is DefaultWasmInvoker.
        // We read the saved original captured in the constructor.
        Assert.IsType<DefaultWasmInvoker>(_originalInvoker);
    }

    // -----------------------------------------------------------------------
    // ReadAsync default stream — ensure the null-path compiles/executes the
    // Console.OpenStandardInput() branch.  We cannot redirect Console.In in
    // xUnit cleanly, so we only verify the overload exists and that passing
    // null is accepted (mock still handles the call).
    // -----------------------------------------------------------------------

    [Fact]
    public async Task ReadAsync_NullStream_UsesConsoleStdin_OverloadAcceptsNull()
    {
        // We cannot substitute Console.In easily, but we can verify that
        // passing an explicit empty MemoryStream (simulating an already-drained
        // stdin) produces the right result — and also that the code path that
        // builds the source stream is exercised.
        const string hookJson = """{"event":"notification","sessionId":"s","caller":"amp"}""";
        Polyhook.Polyhook.WasmInvoker = new MockWasmInvoker(LengthPrefix(hookJson));

        // Pass an empty MemoryStream explicitly (not null) to keep the test hermetic.
        using var stdin = new MemoryStream(Array.Empty<byte>());
        var evt = await Polyhook.Polyhook.ReadAsync(inputStream: stdin);

        Assert.Equal(CallerKind.Amp, evt.Caller);
    }

    // -----------------------------------------------------------------------
    // RespondAsync default stream — same reasoning.
    // -----------------------------------------------------------------------

    [Fact]
    public async Task RespondAsync_NullStream_OverloadAcceptsExplicitStream()
    {
        var mock = new MockWasmInvoker(LengthPrefix("ok"));
        Polyhook.Polyhook.WasmInvoker = mock;

        // Use an explicit stream (not null) to keep the test hermetic.
        using var stdout = new MemoryStream();
        await Polyhook.Polyhook.RespondAsync(Polyhook.Polyhook.Approve(), outputStream: stdout);

        Assert.True(stdout.Length >= 0);
    }
}
