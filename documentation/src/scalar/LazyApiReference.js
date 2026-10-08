// Route component for /api: loads the Scalar standalone script, then hands over
// to the upstream component, which calls window.Scalar.createApiReference on
// mount. Rendering the upstream component only once window.Scalar exists keeps
// its mount-time check valid on a hard load as well as on client-side
// navigation. The script is fetched once per page session and shared by every
// later visit to /api.

import React, { useEffect, useState } from "react";
import Layout from "@theme/Layout";
import ScalarDocusaurus from "@scalar/docusaurus/dist/ScalarDocusaurus";

let scriptPromise = null;

function loadScalarScript(src) {
  if (window.Scalar) {
    return Promise.resolve();
  }
  if (!scriptPromise) {
    scriptPromise = new Promise((resolve, reject) => {
      const script = document.createElement("script");
      script.src = src;
      script.async = true;
      script.onload = () => resolve();
      script.onerror = () => {
        // Forget the failed attempt so the next visit to /api tries again.
        scriptPromise = null;
        script.remove();
        reject(new Error(`Failed to load ${src}`));
      };
      document.head.appendChild(script);
    });
  }
  return scriptPromise;
}

export default function LazyApiReference(props) {
  const { route } = props;
  const [state, setState] = useState(() =>
    typeof window !== "undefined" && window.Scalar ? "ready" : "loading"
  );

  useEffect(() => {
    if (state !== "loading") {
      return undefined;
    }
    let cancelled = false;
    loadScalarScript(route.scriptSrc).then(
      () => !cancelled && setState("ready"),
      () => !cancelled && setState("failed")
    );
    return () => {
      cancelled = true;
    };
  }, [state, route.scriptSrc]);

  if (state === "ready") {
    return <ScalarDocusaurus {...props} />;
  }
  return (
    <Layout>
      {state === "failed" && (
        <p style={{ padding: "2rem" }}>
          The API reference viewer could not be loaded. The OpenAPI
          specification is available at <a href="/hook0-api.json">/hook0-api.json</a>.
        </p>
      )}
    </Layout>
  );
}
