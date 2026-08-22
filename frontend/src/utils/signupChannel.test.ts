import {
  declaredChannel,
  DECLARED_ORIGINS,
  deriveSignupChannel,
  readSignupChannel,
  rememberSignupChannel,
  UNKNOWN_CHANNEL,
  type SignupChannelSource,
  type SignupChannelStorage,
} from './signupChannel';

/** The key the module owns, asserted on from the outside. */
const STORAGE_KEY = 'hook0_signup_channel';

/**
 * A real implementation of the storage contract with the semantics a browser
 * gives (string in, string out, `null` when absent), just without a browser.
 */
function memoryStorage(): SignupChannelStorage & { keys(): string[] } {
  const entries = new Map<string, string>();
  return {
    getItem: (key) => {
      const stored = entries.get(key);
      return stored === undefined ? null : stored;
    },
    setItem: (key, value) => {
      entries.set(key, value);
    },
    keys: () => [...entries.keys()],
  };
}

/** A visitor landing on the application, with only the referrer varying. */
function landingFrom(referrer: string, search = ''): SignupChannelSource {
  return { referrer, search, host: 'app.hook0.com' };
}

/**
 * The grammar `api/src/signup_channel.rs` and the CHECK constraint on
 * `iam.user.signup_channel` both enforce. A label that fails this is stored as
 * `unknown`, so emitting one silently loses the very information this module
 * exists to capture.
 */
const STORED_GRAMMAR =
  /^(unknown|direct|(ads|organic|ai|social|referral|campaign|declared):[a-z0-9][a-z0-9.-]{0,63})$/;

describe('deriveSignupChannel', () => {
  it('reads an ad click from the landing URL', () => {
    expect(deriveSignupChannel(landingFrom('', '?gclid=EAIaIQobChMI'))).toBe('ads:google');
  });

  it('prefers the ad click over a campaign parameter on the same URL', () => {
    expect(deriveSignupChannel(landingFrom('', '?gclid=abc&utm_source=newsletter'))).toBe(
      'ads:google'
    );
  });

  it('reads a tagged campaign', () => {
    expect(deriveSignupChannel(landingFrom('', '?utm_source=Newsletter'))).toBe(
      'campaign:newsletter'
    );
  });

  it('reads the campaign Matomo tags, not only the utm_ family', () => {
    expect(deriveSignupChannel(landingFrom('', '?mtm_source=Newsletter'))).toBe(
      'campaign:newsletter'
    );
    expect(deriveSignupChannel(landingFrom('', '?mtm_campaign=launch-week'))).toBe(
      'campaign:launch-week'
    );
    expect(deriveSignupChannel(landingFrom('', '?pk_campaign=legacy'))).toBe('campaign:legacy');
  });

  it('prefers the source over the campaign name, whichever family tagged it', () => {
    expect(
      deriveSignupChannel(landingFrom('', '?mtm_campaign=launch-week&mtm_source=newsletter'))
    ).toBe('campaign:newsletter');
    expect(
      deriveSignupChannel(landingFrom('', '?mtm_campaign=launch-week&utm_source=partner'))
    ).toBe('campaign:partner');
  });

  it('keeps a campaign tag ahead of the referrer that carried it', () => {
    expect(
      deriveSignupChannel({
        referrer: 'https://www.linkedin.com/feed/',
        search: '?mtm_campaign=launch-week',
        host: 'app.hook0.com',
      })
    ).toBe('campaign:launch-week');
  });

  it('calls a visit with no referrer direct', () => {
    expect(deriveSignupChannel(landingFrom(''))).toBe('direct');
  });

  it('recognises search engines, country domains included', () => {
    expect(deriveSignupChannel(landingFrom('https://www.google.com/'))).toBe('organic:google');
    expect(deriveSignupChannel(landingFrom('https://www.google.co.uk/search?q=webhooks'))).toBe(
      'organic:google'
    );
    expect(deriveSignupChannel(landingFrom('https://duckduckgo.com/'))).toBe('organic:duckduckgo');
    expect(deriveSignupChannel(landingFrom('https://search.brave.com/search?q=hook0'))).toBe(
      'organic:brave'
    );
  });

  it('counts only the search engine itself as organic, not what is hosted beside it', () => {
    expect(deriveSignupChannel(landingFrom('https://news.google.com/articles/x'))).toBe(
      'referral:news.google.com'
    );
    expect(deriveSignupChannel(landingFrom('https://mail.google.com/'))).toBe(
      'referral:mail.google.com'
    );
  });

  it('tells AI assistants apart from the search engine they share a domain with', () => {
    expect(deriveSignupChannel(landingFrom('https://gemini.google.com/app'))).toBe('ai:gemini');
    expect(deriveSignupChannel(landingFrom('https://chatgpt.com/'))).toBe('ai:chatgpt');
    expect(deriveSignupChannel(landingFrom('https://www.perplexity.ai/search'))).toBe(
      'ai:perplexity'
    );
  });

  it('recognises social and community sources', () => {
    expect(deriveSignupChannel(landingFrom('https://www.linkedin.com/feed/'))).toBe(
      'social:linkedin'
    );
    expect(deriveSignupChannel(landingFrom('https://news.ycombinator.com/item?id=1'))).toBe(
      'social:hackernews'
    );
    expect(deriveSignupChannel(landingFrom('https://t.co/abc'))).toBe('social:x');
  });

  it('keeps any other site as a plain referral, host only', () => {
    expect(deriveSignupChannel(landingFrom('https://openalternative.co/hook0?ref=list'))).toBe(
      'referral:openalternative.co'
    );
  });

  it('drops the path, the query string and the fragment of the referrer', () => {
    const channel = deriveSignupChannel(
      landingFrom('https://example.com/users/42/invoices?token=secret#section')
    );
    expect(channel).toBe('referral:example.com');
  });

  it('says unknown for the host the form itself is served from', () => {
    expect(deriveSignupChannel(landingFrom('https://app.hook0.com/login'))).toBe(UNKNOWN_CHANNEL);
  });

  it('names the sibling Hook0 property that handed the visitor over', () => {
    expect(deriveSignupChannel(landingFrom('https://www.hook0.com/pricing'))).toBe(
      'referral:hook0.com'
    );
    expect(deriveSignupChannel(landingFrom('https://documentation.hook0.com/'))).toBe(
      'referral:documentation.hook0.com'
    );
    expect(deriveSignupChannel(landingFrom('https://play.hook0.com/'))).toBe(
      'referral:play.hook0.com'
    );
  });

  it('says unknown for a referrer it cannot parse', () => {
    expect(deriveSignupChannel(landingFrom('not a url'))).toBe(UNKNOWN_CHANNEL);
  });

  it('refuses an oversized referrer instead of parsing it', () => {
    const oversized = `https://example.com/${'a'.repeat(3000)}`;
    expect(deriveSignupChannel(landingFrom(oversized))).toBe(UNKNOWN_CHANNEL);
  });

  it('refuses a host too long for the stored vocabulary', () => {
    const longHost = `${'a'.repeat(70)}.com`;
    expect(deriveSignupChannel(landingFrom(`https://${longHost}/`))).toBe(UNKNOWN_CHANNEL);
  });

  it('refuses a campaign name that is not a slug', () => {
    expect(deriveSignupChannel(landingFrom('', '?utm_source=news letter'))).toBe('direct');
    expect(deriveSignupChannel(landingFrom('', "?utm_source='; DROP TABLE"))).toBe('direct');
  });

  it('only ever emits labels the API and the database accept', () => {
    const sources = [
      landingFrom(''),
      landingFrom('', '?gclid=abc'),
      landingFrom('', '?utm_source=Newsletter'),
      landingFrom('', '?utm_source=<script>alert(1)</script>'),
      landingFrom('', '?mtm_campaign=Launch Week'),
      landingFrom('', `?pk_source=${'a'.repeat(80)}`),
      landingFrom('https://www.google.fr/'),
      landingFrom('https://gemini.google.com/'),
      landingFrom('https://openalternative.co/'),
      landingFrom('https://www.hook0.com/'),
      landingFrom('not a url'),
      landingFrom(`https://${'a'.repeat(70)}.com/`),
      landingFrom('https://xn--n3h.example.com/'),
    ];

    for (const source of sources) {
      expect(deriveSignupChannel(source)).toMatch(STORED_GRAMMAR);
    }
  });
});

describe('declaredChannel', () => {
  it('keeps every answer the form can offer', () => {
    for (const origin of DECLARED_ORIGINS) {
      expect(declaredChannel(origin)).toBe(`declared:${origin}`);
    }
  });

  it('marks a declaration as such, so it is never read as a detection', () => {
    expect(declaredChannel('search')).toBe('declared:search');
    expect(declaredChannel('search')).not.toBe('organic:google');
  });

  it('refuses anything the form did not offer', () => {
    expect(declaredChannel('')).toBe(UNKNOWN_CHANNEL);
    expect(declaredChannel('whatever')).toBe(UNKNOWN_CHANNEL);
    expect(declaredChannel("'; DROP TABLE iam.user; --")).toBe(UNKNOWN_CHANNEL);
  });

  it('only emits labels the API and the database accept', () => {
    for (const origin of [...DECLARED_ORIGINS, 'nonsense', '']) {
      expect(declaredChannel(origin)).toMatch(STORED_GRAMMAR);
    }
  });
});

describe('rememberSignupChannel', () => {
  it('records the channel of the entry page under its own key', () => {
    const storage = memoryStorage();

    rememberSignupChannel(landingFrom('https://www.linkedin.com/feed/'), storage);

    expect(storage.keys()).toEqual([STORAGE_KEY]);
    expect(readSignupChannel(storage)).toBe('social:linkedin');
  });

  it('keeps the first entry when the visitor comes back later in the tab', () => {
    const storage = memoryStorage();

    rememberSignupChannel(landingFrom('https://www.linkedin.com/feed/'), storage);
    rememberSignupChannel(landingFrom('https://app.hook0.com/login'), storage);

    expect(readSignupChannel(storage)).toBe('social:linkedin');
  });

  it('records direct rather than nothing, so a later visit cannot overwrite it', () => {
    const storage = memoryStorage();

    rememberSignupChannel(landingFrom(''), storage);

    expect(readSignupChannel(storage)).toBe('direct');
  });
});

describe('readSignupChannel', () => {
  it('reads unknown when nothing was recorded', () => {
    expect(readSignupChannel(memoryStorage())).toBe(UNKNOWN_CHANNEL);
  });

  it('hands back exactly what was recorded, so the API sees one vocabulary', () => {
    const storage = memoryStorage();

    rememberSignupChannel(landingFrom('https://openalternative.co/hook0'), storage);

    expect(readSignupChannel(storage)).toMatch(STORED_GRAMMAR);
    expect(readSignupChannel(storage)).toBe('referral:openalternative.co');
  });
});
