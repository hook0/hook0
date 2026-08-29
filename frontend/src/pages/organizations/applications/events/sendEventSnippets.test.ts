import fc from 'fast-check';
import { HOOK0_SDKS, type Hook0Sdk, type Hook0SdkStringLiteral } from '@/generated/sdkExamples';
import { HOOK0_CODE_LANGUAGES } from '@/components/Hook0Code';
import { PLAIN_TEXT_LANGUAGES, isPlainTextLanguage } from '@/components/Hook0CodeColouring';
import { execFileSync } from 'node:child_process';
import {
  CURL_PANEL,
  FEATURED_TARGETS,
  FORM_PANEL,
  ICON_TARGETS,
  escapeForLiteral,
  hashForPanel,
  panelFromHash,
  partitionSdks,
  renderCurlSnippet,
  renderSdkSnippets,
  shellSingleQuoted,
  type Hook0CurlValues,
  type Hook0SnippetValues,
} from './sendEventSnippets';

// A C-family literal: the backslash first, exactly as every SDK manifest declares it, so that the
// rendered literal is also a JSON string and can be read back with `JSON.parse`. That is what lets
// a test say "this value did not close the literal early" without knowing the language.
const C_LIKE_LITERAL: Hook0SdkStringLiteral = {
  open: '"',
  close: '"',
  escape: [
    ['\\', '\\\\'],
    ['"', '\\"'],
    ['\n', '\\n'],
    ['\r', '\\r'],
    ['\t', '\\t'],
  ],
};

const SEND_HEAD =
  'client("__HOOK0_API_URL__", "__HOOK0_APPLICATION_ID__", "__HOOK0_TOKEN__");\nlabels = [\n';
const SEND_LABEL = '  ("__HOOK0_LABEL_KEY__", "__HOOK0_LABEL_VALUE__")';
const SEND_TAIL = '\n];\nsend("__HOOK0_EVENT_TYPE__", "__HOOK0_PAYLOAD__", labels);';

function anSdk(overrides: Partial<Hook0Sdk> = {}): Hook0Sdk {
  return {
    target: 'typescript',
    displayName: 'JavaScript / TypeScript',
    packageName: 'hook0-client',
    registry: 'npm',
    version: '2.0.2',
    install: 'npm install hook0-client@2.0.2',
    publishedToRegistry: true,
    send: {
      body: SEND_HEAD + SEND_LABEL + SEND_TAIL,
      label: SEND_LABEL,
      labelSeparator: ',\n',
    },
    verify: { body: 'verify(signature, body, headers, env.HOOK0_SUBSCRIPTION_SECRET);' },
    stringLiteral: C_LIKE_LITERAL,
    ...overrides,
  };
}

const APPLICATION_ID = '2f4e8a1c-9b3d-4c7e-8f10-5a6b7c8d9e0f';

function theForm(overrides: Partial<Hook0SnippetValues> = {}): Hook0SnippetValues {
  return {
    apiUrl: 'https://app.hook0.com/api/v1',
    applicationId: APPLICATION_ID,
    token: 'hk0_tok',
    eventType: 'user.account.created',
    payload: '{"test": true}',
    labels: [{ key: 'user_id', value: '1' }],
    ...overrides,
  };
}

/** What the head of the example reads as once the form's values are in it. */
const RENDERED_HEAD = `client("https://app.hook0.com/api/v1", "${APPLICATION_ID}", "hk0_tok");\nlabels = [\n`;

/** What the tail reads as, the payload's own quotes escaped into the literal that holds it. */
const RENDERED_TAIL = '\n];\nsend("user.account.created", "{\\"test\\": true}", labels);';

/**
 * Text a user could plausibly type into the payload or a label and that a naive renderer breaks on:
 * the closing quote, the backslash, the newline, Rust's raw-string fence, and the `$&` family that
 * a replacement string would read as an instruction rather than as characters.
 */
const hostileText = fc
  .array(
    fc.constantFrom(
      'a',
      'Z',
      '0',
      ' ',
      '"',
      '\\',
      '\n',
      '\r',
      '\t',
      '{',
      '}',
      ':',
      '#',
      "'",
      '`',
      'r#"',
      '"#',
      '$&',
      "$'",
      '$`',
      '$1',
      '$$',
      'é'
    ),
    { maxLength: 12 }
  )
  .map((parts) => parts.join(''));

describe('escapeForLiteral', () => {
  it('leaves a value with nothing to escape alone', () => {
    expect(escapeForLiteral(C_LIKE_LITERAL, 'user.account.created')).toBe('user.account.created');
  });

  it('escapes the backslash before the quotes it would otherwise have introduced', () => {
    expect(escapeForLiteral(C_LIKE_LITERAL, '\\"')).toBe('\\\\\\"');
  });

  it('never lets a value close the literal it is put in', () => {
    fc.assert(
      fc.property(hostileText, (raw) => {
        const literal =
          C_LIKE_LITERAL.open + escapeForLiteral(C_LIKE_LITERAL, raw) + C_LIKE_LITERAL.close;
        expect(JSON.parse(literal)).toBe(raw);
      }),
      { numRuns: 500, seed: 20260827 }
    );
  });
});

describe('renderSdkSnippets', () => {
  it('puts the form in the example and leaves no marker behind', () => {
    const rendered = renderSdkSnippets(anSdk(), theForm());

    expect(rendered.send).toBe(`${RENDERED_HEAD}  ("user_id", "1")${RENDERED_TAIL}`);
    expect(rendered.send).not.toContain('__HOOK0_');
  });

  it('renders the install command and the verify example too', () => {
    const rendered = renderSdkSnippets(
      anSdk({ install: 'go get github.com/hook0/hook0-go/v2 # __HOOK0_APPLICATION_ID__' }),
      theForm()
    );

    expect(rendered.install).toBe(`go get github.com/hook0/hook0-go/v2 # ${APPLICATION_ID}`);
    expect(rendered.verify).toBe(
      'verify(signature, body, headers, env.HOOK0_SUBSCRIPTION_SECRET);'
    );
  });

  it('leaves the container empty when the form carries no label', () => {
    const rendered = renderSdkSnippets(anSdk(), theForm({ labels: [] }));

    expect(rendered.send).toBe(RENDERED_HEAD + RENDERED_TAIL);
    expect(rendered.send).not.toContain('__HOOK0_');
  });

  it('joins the repetitions with the separator the SDK declares, and only between them', () => {
    const rendered = renderSdkSnippets(
      anSdk(),
      theForm({
        labels: [
          { key: 'a', value: '1' },
          { key: 'b', value: '2' },
          { key: 'c', value: '3' },
        ],
      })
    );

    expect(rendered.send).toBe(
      `${RENDERED_HEAD}  ("a", "1"),\n  ("b", "2"),\n  ("c", "3")${RENDERED_TAIL}`
    );
  });

  it('escapes a hostile payload rather than letting it break out of its literal', () => {
    const rendered = renderSdkSnippets(anSdk(), theForm({ payload: '{"a": "b\\c"}\n' }));

    expect(rendered.send).toContain('"{\\"a\\": \\"b\\\\c\\"}\\n"');
  });

  it('escapes label keys and values, not only the payload', () => {
    const rendered = renderSdkSnippets(
      anSdk(),
      theForm({ labels: [{ key: 'we"ird', value: 'v\\1' }] })
    );

    expect(rendered.send).toContain('("we\\"ird", "v\\\\1")');
  });

  it('hands back a value spelling `$&` as those characters', () => {
    const rendered = renderSdkSnippets(anSdk(), theForm({ labels: [{ key: '$&', value: "$'" }] }));

    expect(rendered.send).toContain(`("$&", "$'")`);
  });

  it('renders two labels, one of them a key no language would let you write bare', () => {
    // Two rather than one: a separator placed inside the region instead of between the repetitions,
    // an indentation taken from the first copy only, and a region spliced back at the wrong offset
    // all render correctly with a single label and wrongly with two. And `env-name` is the key that
    // matters — it is not an identifier, so a language quoting its keys only when they look like
    // one breaks on it, while `user_id` never would.
    const rendered = renderSdkSnippets(
      anSdk(),
      theForm({
        labels: [
          { key: 'env-name', value: 'pro"d\\stage\nnext' },
          { key: 'user_id', value: '1' },
        ],
      })
    );

    expect(rendered.send).toBe(
      `${RENDERED_HEAD}  ("env-name", "pro\\"d\\\\stage\\nnext"),\n  ("user_id", "1")${RENDERED_TAIL}`
    );
    // Both copies sit on the indentation the region carried, rather than the second losing it.
    expect(rendered.send).toContain('\n  ("env-name"');
    expect(rendered.send).toContain('\n  ("user_id"');
  });

  it('does not read a marker the user typed as a marker of its own', () => {
    const rendered = renderSdkSnippets(
      anSdk(),
      theForm({ payload: '__HOOK0_TOKEN__', labels: [{ key: '__HOOK0_PAYLOAD__', value: 'x' }] })
    );

    // The payload reaches the snippet as the word the user typed, and the label likewise: neither
    // is substituted a second time, so no value of the form leaks into another's place.
    expect(rendered.send).toContain('send("user.account.created", "__HOOK0_TOKEN__"');
    expect(rendered.send).toContain('("__HOOK0_PAYLOAD__", "x")');
    expect(rendered.send).not.toContain('"{\\"test\\": true}"');
  });

  it('refuses an example whose label region it cannot find', () => {
    const broken = anSdk({
      send: { body: 'labels = [];', label: SEND_LABEL, labelSeparator: ',\n' },
    });

    expect(() => renderSdkSnippets(broken, theForm())).toThrow(
      'the typescript send example does not carry its label region'
    );
  });

  it('refuses an example whose label region is empty', () => {
    const broken = anSdk({
      target: 'zig',
      send: { body: SEND_HEAD + SEND_TAIL, label: '', labelSeparator: ',\n' },
    });

    expect(() => renderSdkSnippets(broken, theForm())).toThrow(
      'the zig send example does not carry its label region'
    );
  });

  it('leaves no marker behind whatever the user types', () => {
    fc.assert(
      fc.property(
        hostileText,
        hostileText,
        hostileText,
        fc.array(fc.tuple(hostileText, hostileText), { maxLength: 4 }),
        (payload, eventType, token, labelPairs) => {
          const rendered = renderSdkSnippets(
            anSdk(),
            theForm({
              payload,
              eventType,
              token,
              labels: labelPairs.map(([key, value]) => ({ key, value })),
            })
          );

          // The values above cannot spell a marker, so any left on screen is one the renderer
          // failed to substitute rather than one the user asked for.
          expect(rendered.send).not.toContain('__HOOK0_');
          expect(rendered.install).not.toContain('__HOOK0_');
          expect(rendered.verify).not.toContain('__HOOK0_');
        }
      ),
      { numRuns: 500, seed: 20260827 }
    );
  });

  it('splices the label region back exactly where it was taken from', () => {
    fc.assert(
      fc.property(fc.array(hostileText, { maxLength: 6 }), (labelValues) => {
        const labels = labelValues.map((value, index) => ({ key: `k${index}`, value }));
        const rendered = renderSdkSnippets(anSdk(), theForm({ labels }));

        const expected = labels
          .map(({ key, value }) => `  ("${key}", "${escapeForLiteral(C_LIKE_LITERAL, value)}")`)
          .join(',\n');
        expect(rendered.send).toBe(RENDERED_HEAD + expected + RENDERED_TAIL);
      }),
      { numRuns: 500, seed: 20260827 }
    );
  });
});

const EVENT_ID = '0d5c6d2e-6c1a-4f4c-9f7a-2b3c4d5e6f70';
const OCCURRED_AT = '2026-08-27T09:15:00.000Z';

/** How long the blocks that run a real shell are given; each property run spawns one. */
const SHELL_TIMEOUT_MS = 60_000;

function theCall(overrides: Partial<Hook0CurlValues> = {}): Hook0CurlValues {
  return {
    ...theForm(),
    eventId: EVENT_ID,
    occurredAt: OCCURRED_AT,
    ...overrides,
  };
}

/**
 * The arguments a POSIX shell hands to `curl` when the rendered command is pasted into it.
 *
 * Read by running the command in a real shell rather than by taking the quoting apart here: what is
 * measured is exactly what `/bin/sh` makes of the text, and a second implementation of the quoting
 * rule written in this file would agree with the first one's mistakes. `curl` is replaced by a
 * function printing its own arguments, so nothing leaves the machine and the arguments are read as
 * the shell built them — separated by a NUL, the one byte an argument cannot hold.
 *
 * A command the shell cannot parse throws here, which is the failure this whole block is about.
 */
function shellArgumentsOf(command: string): string[] {
  const script = `curl() { for a in "$@"; do printf '%s\\0' "$a"; done; }\n${command}\n`;
  const printed = execFileSync('/bin/sh', ['-c', script], { encoding: 'utf8' });
  const argumentsPrinted = printed.split('\0');
  // `printf` writes a NUL after each argument, so the split leaves an empty tail.
  argumentsPrinted.pop();
  return argumentsPrinted;
}

/** What the command posts, as the API would read it. */
function bodyPostedBy(command: string): Record<string, unknown> {
  const argumentsGiven = shellArgumentsOf(command);
  const at = argumentsGiven.indexOf('-d');
  if (at < 0) {
    throw new Error(`the command carries no body: ${command}`);
  }
  return JSON.parse(argumentsGiven[at + 1]) as Record<string, unknown>;
}

describe('shellSingleQuoted', () => {
  it('leaves a value with nothing to quote readable', () => {
    expect(shellSingleQuoted('user.account.created')).toBe(`'user.account.created'`);
  });

  it('splices the quote back outside the quoting rather than letting it end the argument', () => {
    expect(shellSingleQuoted("O'Brien")).toBe(`'O'\\''Brien'`);
  });

  it(
    'hands a shell back the value it was given, whatever the value',
    () => {
      fc.assert(
        fc.property(hostileText, (raw) => {
          const [read] = shellArgumentsOf(`curl ${shellSingleQuoted(raw)}`);
          expect(read).toBe(raw);
        }),
        { numRuns: 60, seed: 20260827 }
      );
    },
    SHELL_TIMEOUT_MS
  );
});

describe('renderCurlSnippet', () => {
  it('posts the event the form describes', () => {
    const body = bodyPostedBy(renderCurlSnippet(theCall()));

    expect(body).toEqual({
      application_id: APPLICATION_ID,
      event_id: EVENT_ID,
      event_type: 'user.account.created',
      labels: { user_id: '1' },
      occurred_at: OCCURRED_AT,
      payload: '{"test": true}',
      payload_content_type: 'application/json',
    });
  });

  it('addresses the endpoint the page is talking to, and carries the token', () => {
    const argumentsGiven = shellArgumentsOf(renderCurlSnippet(theCall()));

    expect(argumentsGiven.slice(0, 3)).toEqual([
      '-X',
      'POST',
      'https://app.hook0.com/api/v1/event',
    ]);
    expect(argumentsGiven).toContain('Authorization: Bearer hk0_tok');
    expect(argumentsGiven).toContain('Content-Type: application/json');
  });

  it('survives an apostrophe in the payload, which used to end the argument early', () => {
    const payload = '{"name": "O\'Brien"}';
    const body = bodyPostedBy(renderCurlSnippet(theCall({ payload })));

    expect(body.payload).toBe(payload);
  });

  it('survives an apostrophe in a label and in the token', () => {
    const command = renderCurlSnippet(
      theCall({ labels: [{ key: "l'cle", value: "l'evenement" }], token: "hk0'tok" })
    );

    expect(bodyPostedBy(command).labels).toEqual({ "l'cle": "l'evenement" });
    expect(shellArgumentsOf(command)).toContain("Authorization: Bearer hk0'tok");
  });

  it(
    'posts what the reader typed, whatever the reader typed',
    () => {
      fc.assert(
        fc.property(
          hostileText,
          fc.array(fc.tuple(hostileText, hostileText), { maxLength: 3 }),
          (payload, labelPairs) => {
            const labels = labelPairs.map(([key, value]) => ({ key, value }));
            const body = bodyPostedBy(renderCurlSnippet(theCall({ payload, labels })));

            expect(body.payload).toBe(payload);
            // Compared as the object the request carries rather than row by row, so two rows
            // sharing a key are read the way JSON keeps them.
            const expected: Record<string, string> = {};
            for (const { key, value } of labels) {
              expected[key] = value;
            }
            expect(body.labels).toEqual(expected);
          }
        ),
        { numRuns: 60, seed: 20260827 }
      );
    },
    SHELL_TIMEOUT_MS
  );
});

describe('panelFromHash', () => {
  const sdks = [anSdk(), anSdk({ target: 'rust' }), anSdk({ target: 'zig' })];

  it('opens the form on an empty fragment', () => {
    expect(panelFromHash('', sdks)).toBe(FORM_PANEL);
  });

  it('opens the form on a fragment naming nothing', () => {
    expect(panelFromHash('#cobol', sdks)).toBe(FORM_PANEL);
  });

  it('opens cURL, which no SDK carries', () => {
    expect(panelFromHash('#curl', sdks)).toBe(CURL_PANEL);
  });

  it('opens a language by its registry name', () => {
    expect(panelFromHash('#zig', sdks)).toBe('zig');
  });

  it('keeps the links that were shared before the screen carried eleven languages', () => {
    expect(panelFromHash('#js', sdks)).toBe('typescript');
    expect(panelFromHash('#rust', sdks)).toBe('rust');
  });

  it('drops a legacy fragment whose language is gone rather than opening the wrong one', () => {
    expect(panelFromHash('#js', [anSdk({ target: 'rust' })])).toBe(FORM_PANEL);
  });
});

describe('hashForPanel', () => {
  it('gives the form no fragment, since it is what an unfragmented URL opens', () => {
    expect(hashForPanel(FORM_PANEL)).toBe('');
  });

  it('names every other panel', () => {
    expect(hashForPanel(CURL_PANEL)).toBe('#curl');
    expect(hashForPanel('python')).toBe('#python');
  });
});

describe('partitionSdks', () => {
  const sdks = [
    anSdk(),
    anSdk({ target: 'python' }),
    anSdk({ target: 'go' }),
    anSdk({ target: 'rust' }),
  ];

  it('puts the featured targets in front, in the order asked for', () => {
    const { featured, others } = partitionSdks(sdks, ['python', 'typescript']);

    expect(featured.map((sdk) => sdk.target)).toEqual(['python', 'typescript']);
    expect(others.map((sdk) => sdk.target)).toEqual(['go', 'rust']);
  });

  it('keeps every SDK when none is featured', () => {
    const { featured, others } = partitionSdks(sdks, []);

    expect(featured).toEqual([]);
    expect(others).toHaveLength(4);
  });

  it('refuses to promise a tab no SDK answers to', () => {
    expect(() => partitionSdks(sdks, ['cobol'])).toThrow(
      'no SDK targets cobol, so it cannot be shown as its own tab'
    );
  });
});

/**
 * The same functions, held against the registry the screen is actually handed.
 *
 * Everything above runs on fixtures, which is what keeps it about the rendering rather than about
 * today's eleven SDKs. These four are the opposite: they are the couplings between this screen and
 * the generated artefact that nothing else would catch — a target renamed on one side only, a
 * language nobody thought to colour, an example whose label region the generator lost. Each of them
 * would otherwise surface as a dead screen or a wrong snippet in front of a reader.
 */
describe('the registry the screen is given', () => {
  it('carries the languages promised their own tab', () => {
    // `partitionSdks` throws on a featured target no SDK answers to, and the screen calls it while
    // it is being set up: getting this wrong takes the whole page down, onboarding included.
    expect(() => partitionSdks(HOOK0_SDKS, FEATURED_TARGETS)).not.toThrow();
  });

  it('still answers to the fragments shared before the rename', () => {
    // `#js` names no target: it survives only because the module maps it onto one that exists. A
    // target renamed without touching that map turns every link in a ticket into the plain form.
    const opened = panelFromHash('#js', HOOK0_SDKS);
    expect(HOOK0_SDKS.map((sdk) => sdk.target)).toContain(opened);
  });

  it('carries a mark for every language, and no mark for a language that is gone', () => {
    // Bidirectional, like the guard on the Rust side: a target with no icon would reach the picker
    // unmarked and nothing would say so, and an icon left behind after an SDK is dropped would be
    // an import nothing renders. The component types its own map by `ICON_TARGETS`, so agreeing
    // with the registry here is the whole of it.
    const declared = [...ICON_TARGETS].sort();
    const registered = HOOK0_SDKS.map((sdk) => sdk.target).sort();

    expect(declared).toEqual(registered);
  });

  it('colours every shown language it does not declare plain, and declares plain only what it shows', () => {
    // The screen hands each target to the code block, which colours it through a CodeMirror grammar
    // unless it is declared plain. Membership in `HOOK0_CODE_LANGUAGES` is not colourability — that
    // list is the accepted-name list and it holds `zig`, which by design has no grammar — so the
    // property is held against the plain-text declaration instead. That declaration is what
    // `Hook0Code.vue` builds its grammar map from: a coloured language owes a loader and a plain one
    // owes `null`, a coupling `vue-tsc` holds through the map's type. The runtime half is here.
    const shown = new Set<string>(HOOK0_SDKS.map((sdk) => sdk.target));

    // The classification, exercised across the whole accepted-name registry so both arms of the
    // decision are visited: exactly the declared plain-text languages come back plain, and at least
    // one language is left to colour.
    const classifiedPlain = HOOK0_CODE_LANGUAGES.filter((language) =>
      isPlainTextLanguage(language)
    );
    const classifiedColoured = HOOK0_CODE_LANGUAGES.filter(
      (language) => !isPlainTextLanguage(language)
    );
    expect(classifiedPlain).toEqual([...PLAIN_TEXT_LANGUAGES]);
    expect(classifiedColoured.length).toBeGreaterThan(0);

    // Held against the registry in both directions. A plain declaration for a language nobody shows
    // is stale and would hide a real regression the day that name returned; and a shown language
    // that is not declared plain is one the block is expected to colour, of which there is at least
    // one or this screen colours nothing.
    for (const plain of PLAIN_TEXT_LANGUAGES) {
      expect(shown.has(plain)).toBe(true);
    }
    const shownColoured = [...shown].filter(
      (target) => !(PLAIN_TEXT_LANGUAGES as readonly string[]).includes(target)
    );
    expect(shownColoured.length).toBeGreaterThan(0);
  });

  it('renders every declared SDK, whatever the reader types', () => {
    fc.assert(
      fc.property(
        hostileText,
        fc.array(fc.tuple(hostileText, hostileText), { maxLength: 3 }),
        (payload, labelPairs) => {
          for (const sdk of HOOK0_SDKS) {
            const rendered = renderSdkSnippets(sdk, {
              ...theForm(),
              payload,
              labels: labelPairs.map(([key, value]) => ({ key, value })),
            });

            expect(rendered.install).not.toContain('__HOOK0_');
            expect(rendered.send).not.toContain('__HOOK0_');
            expect(rendered.verify).not.toContain('__HOOK0_');

            // What the language's own quoting rule is worth once a hostile value has gone through
            // it: the value must still be inside the literal it was put in, and be readable back.
            const literal = sdk.stringLiteral;
            expect(rendered.send).toContain(
              literal.open + escapeForLiteral(literal, payload) + literal.close
            );
          }
        }
      ),
      { numRuns: 100, seed: 20260827 }
    );
  });
});
