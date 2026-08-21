// What holds for every input, rather than for the ones a case happened to pick.
//
// Three things are checked here. A retry schedule never spends more than the policy that produced it
// allows, whichever way the randomness fell. Reading a signature header answers with the one failure
// this package declares, whatever text reached the endpoint, and never with anything else. And a
// value read out of a document the API could answer is written back as the value that was read.
//
// The search is written here rather than taken from a tool: this package installs nothing at runtime
// and its suite reaches for as little as it can. A fixed seed, a bounded number of draws, and the
// counter-examples worth keeping committed under `regressions/` so they run as ordinary cases on
// every pipeline. A failing draw is one somebody can reproduce by running the suite again rather
// than one that goes away on a retry.

using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using System.Text.Json.Nodes;
using Hook0.Generated;
using Xunit;

namespace Hook0.Tests;

/// <summary>What holds whatever the input.</summary>
public sealed class PropertyTests
{
    /// <summary>What the draws are made from. Fixed, so the suite explores the same inputs everywhere.</summary>
    private const int Seed = 20_260_814;

    /// <summary>How many draws each property makes. Bounded, so a pipeline can never be held by one.</summary>
    private const int Draws = 200;

    /// <summary>
    /// How far two sums of the same numbers may sit apart before the difference is a defect rather
    /// than the order they were added in.
    /// </summary>
    private const double Rounding = 1e-9;

    /// <summary>The bounds a drawn policy is built inside.</summary>
    private const int MaxDrawnAttempts = 64;
    private const double MaxDrawnSeconds = 10;
    private const double MaxDrawnBudget = 60;

    /// <summary>Longest header a draw builds.</summary>
    private const int MaxDrawnHeader = 96;

    /// <summary>How many mutations each committed document is put through.</summary>
    private const int Mutations = 4;

    /// <summary>The pieces a signature is made of, put together every way a sender that is not Hook0 might.</summary>
    private static readonly string[] Pieces =
        ["t", "v0", "v1", "h", "=", ",", "0", "9", "zz", "abc", "x-event-id", "1800000000", "-1", "\"", " ", ".", "{", "}"];

    private readonly Random _random = new(Seed);

    [Fact]
    public void ARetryScheduleStaysWithinEveryBoundOfItsPolicy()
    {
        List<(RetryPolicy Policy, IReadOnlyList<double> Draws)> cases = [];

        foreach (JsonNode kept in Corpus.Regressions("retry_policies"))
        {
            JsonArray written = kept.AsArray();
            cases.Add((
                new RetryPolicy
                {
                    MaxAttempts = written[0]!.GetValue<int>(),
                    InitialBackoff = TimeSpan.FromSeconds(written[1]!.GetValue<double>()),
                    MaxBackoff = TimeSpan.FromSeconds(written[2]!.GetValue<double>()),
                    MaxTotalDelay = TimeSpan.FromSeconds(written[3]!.GetValue<double>()),
                },
                [.. written[4]!.AsArray().Select(Unusable)]));
        }

        for (int drawn = 0; drawn < Draws; drawn++)
        {
            cases.Add(DrawnPolicy());
        }

        foreach ((RetryPolicy policy, IReadOnlyList<double> draws) in cases)
        {
            HoldsFor(policy, draws);
        }
    }

    [Fact]
    public void ReadingASignatureAnswersWithTheOneFailureThisPackageDeclares()
    {
        List<string> headers =
            [.. Corpus.Regressions("signatures").Select(kept => kept.GetValue<string>())];
        for (int drawn = 0; drawn < Draws; drawn++)
        {
            headers.Add(DrawnHeader());
        }

        foreach (string header in headers)
        {
            Signature read;
            try
            {
                read = Signature.Parse(header);
            }
            catch (SignatureException)
            {
                continue;
            }

            // Parsing answered, so verifying has to answer the same way: a header that reads must not
            // find a way to fail that a caller cannot name.
            Assert.NotNull(read);
            try
            {
                Webhooks.VerifyWebhookSignatureWithCurrentTime(
                    header,
                    [],
                    [],
                    "a-subscription-secret",
                    TimeSpan.FromSeconds(300),
                    DateTimeOffset.UnixEpoch);
            }
            catch (SignatureException)
            {
                continue;
            }
        }
    }

    [Fact]
    public void AGeneratedTypeReadsBackWhatItWrote()
    {
        IReadOnlyList<JsonNode> documents = Corpus.Regressions("documents");
        List<JsonNode> cases = [.. documents];
        foreach (JsonNode document in documents)
        {
            for (int mutation = 0; mutation < Mutations; mutation++)
            {
                cases.Add(Mutated(document));
            }
        }

        foreach (Type declared in DeclaredModels())
        {
            foreach (JsonNode document in cases)
            {
                object? read;
                try
                {
                    read = JsonSerializer.Deserialize(document.ToJsonString(), declared, Runtime.ReadingOptions);
                }
                catch (JsonException)
                {
                    continue;
                }

                if (read is null)
                {
                    continue;
                }

                string written = JsonSerializer.Serialize(read, declared, Runtime.WritingOptions);
                object? again = JsonSerializer.Deserialize(written, declared, Runtime.ReadingOptions);

                // Compared as what travels rather than as two objects: a member carrying a
                // collection is a different collection every time it is read, and comparing those
                // would say which reference was built rather than which value was carried.
                Assert.NotNull(again);
                Assert.Equal(written, JsonSerializer.Serialize(again, declared, Runtime.WritingOptions));
            }
        }
    }

    /// <summary>
    /// Every record the generator wrote, found by looking at what it wrote. Nothing lists the types
    /// here: a schema the document adds joins this property the moment the generated files carry it.
    /// </summary>
    private static IReadOnlyList<Type> DeclaredModels() =>
    [
        .. typeof(ProblemException).Assembly
            .GetExportedTypes()
            .Where(declared => declared.Namespace == typeof(ProblemException).Namespace)
            .Where(declared => declared.GetMethod("<Clone>$", BindingFlags.Public | BindingFlags.Instance) is not null)
            .OrderBy(declared => declared.Name, StringComparer.Ordinal),
    ];

    private static void HoldsFor(RetryPolicy policy, IReadOnlyList<double> draws)
    {
        IReadOnlyList<TimeSpan> delays = policy.Delays(draws);
        double budget = Math.Max(policy.MaxTotalDelay.TotalSeconds, 0);

        Assert.True(policy.Attempts >= 1);
        Assert.True(policy.Attempts <= RetryPolicy.MaxAttemptsCap);
        Assert.True(delays.Count <= policy.Attempts - 1);
        Assert.True(delays.Sum(delay => delay.TotalSeconds) <= budget + Rounding);

        for (int index = 0; index < delays.Count; index++)
        {
            Assert.True(delays[index] >= TimeSpan.Zero);
            Assert.True(
                delays[index].TotalSeconds <= policy.BackoffCeiling(index + 1).TotalSeconds + Rounding);
            Assert.True(
                delays[index].TotalSeconds <= Math.Max(policy.MaxBackoff.TotalSeconds, 0) + Rounding);
        }

        // A schedule never hurries up as it goes: the ceiling of a retry never sits below the one
        // before it.
        List<double> ceilings =
            [.. Enumerable.Range(1, policy.Attempts).Select(retry => policy.BackoffCeiling(retry).TotalSeconds)];
        Assert.Equal(ceilings.OrderBy(ceiling => ceiling), ceilings);
    }

    /// <summary>A draw that is no draw at all, which has to make the client wait longer, never less.</summary>
    private static double Unusable(JsonNode? drawn) => drawn?.GetValueKind() switch
    {
        JsonValueKind.String => drawn.GetValue<string>() switch
        {
            "nan" => double.NaN,
            "infinity" => double.PositiveInfinity,
            "-infinity" => double.NegativeInfinity,
            _ => 1,
        },
        JsonValueKind.Number => drawn.GetValue<double>(),
        _ => 1,
    };

    private (RetryPolicy Policy, IReadOnlyList<double> Draws) DrawnPolicy()
    {
        double[] draws = new double[_random.Next(0, 9)];
        for (int index = 0; index < draws.Length; index++)
        {
            draws[index] = (_random.NextDouble() * 2) - 0.5;
        }

        return (
            new RetryPolicy
            {
                MaxAttempts = _random.Next(-4, MaxDrawnAttempts + 1),
                InitialBackoff = TimeSpan.FromSeconds(_random.NextDouble() * MaxDrawnSeconds),
                MaxBackoff = TimeSpan.FromSeconds(_random.NextDouble() * MaxDrawnSeconds),
                MaxTotalDelay = TimeSpan.FromSeconds(_random.NextDouble() * MaxDrawnBudget),
            },
            draws);
    }

    private string DrawnHeader()
    {
        int length = _random.Next(0, MaxDrawnHeader + 1);
        System.Text.StringBuilder written = new(length);
        for (int index = 0; index < length; index++)
        {
            written.Append(Pieces[_random.Next(Pieces.Length)]);
        }

        return written.ToString();
    }

    /// <summary>A document with one of its members taken away, replaced, or buried inside something.</summary>
    private JsonNode Mutated(JsonNode document)
    {
        JsonObject read = document.AsObject();
        if (read.Count == 0)
        {
            return document.DeepClone();
        }

        JsonObject mutated = document.DeepClone().AsObject();
        string key = read.Select(member => member.Key).ElementAt(_random.Next(read.Count));

        switch (_random.Next(0, 4))
        {
            case 0:
                mutated.Remove(key);
                break;
            case 1:
                mutated[key] = _random.Next(0, 1001);
                break;
            case 2:
                mutated[key] = new JsonArray(mutated[key]?.DeepClone());
                break;
            default:
                mutated[key] = null;
                break;
        }

        return mutated;
    }
}
