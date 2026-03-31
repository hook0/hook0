import React from 'react';
import Head from '@docusaurus/Head';

function classifyPage(permalink) {
  if (permalink.startsWith('/tutorials/') || permalink.startsWith('/how-to-guides/')) {
    return 'HowTo';
  }
  if (
    permalink.startsWith('/explanation/') ||
    permalink.startsWith('/concepts/') ||
    permalink.startsWith('/reference/')
  ) {
    return 'TechArticle';
  }
  return 'Article';
}

function extractSteps(toc) {
  return toc
    .filter((entry) => entry.level === 2)
    .map((entry, i) => ({
      '@type': 'HowToStep',
      position: i + 1,
      name: entry.value,
      url: `#${entry.id}`,
    }));
}

function extractFAQ(toc) {
  return toc
    .filter((entry) => entry.value.endsWith('?'))
    .map((entry) => ({
      '@type': 'Question',
      name: entry.value,
      acceptedAnswer: {
        '@type': 'Answer',
        text: `See section: ${entry.value}`,
      },
    }));
}

function isIndexPage(permalink) {
  return permalink === '/' || permalink.endsWith('/index') || /\/[^/]+\/$/.test(permalink);
}

export default function SchemaOrg({title, description, permalink, frontMatter, toc}) {
  if (isIndexPage(permalink)) {
    return null;
  }

  const siteUrl = 'https://documentation.hook0.com';
  const fullUrl = `${siteUrl}${permalink}`;
  const pageType = classifyPage(permalink);

  const schema = {
    '@context': 'https://schema.org',
    '@type': pageType,
    headline: title,
    description: description || '',
    url: fullUrl,
    publisher: {
      '@type': 'Organization',
      name: 'Hook0',
      url: 'https://www.hook0.com',
    },
  };

  if (frontMatter.keywords) {
    schema.keywords = Array.isArray(frontMatter.keywords)
      ? frontMatter.keywords.join(', ')
      : frontMatter.keywords;
  }

  if (pageType === 'HowTo' && toc.length > 0) {
    const steps = extractSteps(toc);
    if (steps.length > 0) {
      schema.step = steps;
    }
  }

  const faqItems = extractFAQ(toc);
  if (faqItems.length > 0) {
    schema.mainEntity = faqItems;
    // Add separate FAQPage schema
    const faqSchema = {
      '@context': 'https://schema.org',
      '@type': 'FAQPage',
      mainEntity: faqItems,
    };

    return (
      <Head>
        <script type="application/ld+json">{JSON.stringify(schema)}</script>
        <script type="application/ld+json">{JSON.stringify(faqSchema)}</script>
      </Head>
    );
  }

  return (
    <Head>
      <script type="application/ld+json">{JSON.stringify(schema)}</script>
    </Head>
  );
}
