import React from 'react';
import Layout from '@theme-original/DocItem/Layout';
import {useDoc} from '@docusaurus/plugin-content-docs/client';
import SchemaOrg from '../SchemaOrg';

export default function LayoutWrapper(props) {
  const doc = useDoc();

  return (
    <>
      <SchemaOrg
        title={doc.metadata.title}
        description={doc.metadata.description}
        permalink={doc.metadata.permalink}
        frontMatter={doc.frontMatter}
        toc={doc.toc}
      />
      <Layout {...props} />
    </>
  );
}
