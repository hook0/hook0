// Per-page strings for hook0-vs-convoy (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon.
// SSPL pour Hook0 = « code source ouvert (SSPL-1.0) ». Convoy = MPL-2.0 (OSI), donc « open source » OK pour Convoy.
module.exports = {
  pageTitle: 'Hook0 vs Convoy, comparaison des plateformes webhook | Hook0',
  pageDescription: 'Tous deux en code source ouvert, tous deux sur PostgreSQL. Compare Hook0 (Rust, SSPL-1.0, cloud managé) et Convoy (Go, MPL-2.0, auto-hébergé uniquement). Features et compromis côte à côte.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Hook0 vs Convoy',
    titleAccent: 'Même problème, compromis différents',
    subtitle: 'Tous deux en code source ouvert. Tous deux sur PostgreSQL. Mais les ressemblances s\'arrêtent là, Rust vs Go, cloud managé vs auto-hébergé uniquement, SSPL-1.0 vs MPL-2.0. Cette page décortique ce qui compte vraiment quand tu choisis pour la prod.',
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
  },
  differentiators: {
    eyebrow: 'Pourquoi Hook0',
    h2: 'Différences clés',
    cards: [
      { title: 'Cloud managé vs auto-hébergé uniquement', body: 'Convoy est auto-hébergé uniquement. Pas de cloud managé, point. Tu l\'exécutes, tu le maintiens. Hook0 te laisse choisir, soit le cloud managé (hébergé en Europe), soit l\'auto-hébergement gratuit via Docker ou Kubernetes.' },
      { title: 'Rust vs Go', body: 'Hook0 est écrit en Rust. Pas de garbage collector, donc pas de pauses GC, moins de mémoire utilisée et une latence plus prévisible sous charge. Convoy est écrit en Go, débit correct mais avec garbage collection. À haut volume, l\'écart se voit sur les latences en queue.' },
      { title: 'SSPL vs MPL-2.0', body: 'Convoy utilise MPL-2.0. Très permissive, aucune restriction sur la redistribution. Hook0 utilise SSPL-1.0, la totalité du code source est disponible, mais les fournisseurs cloud ne peuvent pas la revendre comme service concurrent. Les deux sont en code source ouvert. La différence porte sur ce que les tiers peuvent faire avec le code.' },
      { title: 'Hébergement européen vs infra DIY', body: 'Le cloud Hook0 fait tourner son plan de données en France chez Clever Cloud (CDN Cloudflare US divulgué dans le <a href="/fr/accord-traitement-donnees">DPA</a>), conçu pour la conformité RGPD dès le départ. Avec Convoy, tu choisis ta localisation d\'hébergement, mais tu prends aussi tout le stack ops, monitoring, backups, scaling, uptime. Pas d\'option managée, tout est sur ton dos.' },
    ],
  },
  comparison: {
    eyebrow: 'Comparaison de fonctionnalités',
    h2: 'Côte à côte',
    headers: { feature: 'Fonctionnalité', hook0: 'Hook0', convoy: 'Convoy' },
    rows: [
      { feature: 'Licence', hook0Html: 'SSPL-1.0 (source intégrale disponible)', convoyHtml: 'MPL-2.0' },
      { feature: 'Langage', hook0Html: 'Rust', convoyHtml: 'Go' },
      { feature: 'Base de données', hook0Html: 'PostgreSQL seulement', convoyHtml: 'PostgreSQL + Redis' },
      { feature: 'Cloud managé', hook0Html: 'Oui (Clever Cloud FR, CDN Cloudflare US)', convoyHtml: 'Non' },
      { feature: 'Auto-hébergement', hook0Html: 'Gratuit (Docker / K8s)', convoyHtml: 'Oui (seule option)' },
      { feature: 'Tier gratuit', hook0Html: 'Oui (cloud)', convoyHtml: 'N/A (auto-hébergé uniquement)' },
      { feature: 'Signatures HMAC', hook0Html: 'Oui', convoyHtml: 'Oui' },
      { feature: 'Logique de relances', hook0Html: 'Configurable 2-phases (rapide + lent, defaults intelligents)', convoyHtml: 'Configurable' },
      { feature: 'Dépôt principal', hook0Html: '<a href="https://github.com/hook0/hook0" class="underline">GitHub</a> + <a href="https://gitlab.com/hook0/hook0" class="underline">GitLab</a>', convoyHtml: '<a href="https://github.com/frain-dev/convoy" class="underline">GitHub</a> (~2,8k stars)' },
      { feature: 'Financement', hook0Html: '100% bootstrappé', convoyHtml: 'VC-backed (Frain Technologies)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    items: [
      { q: 'Convoy est-il entièrement en code source ouvert ?', a: 'Oui. Convoy utilise la licence MPL-2.0, Hook0 utilise SSPL-1.0. Les deux publient leur code source intégral. La différence pratique tient à la redistribution, MPL-2.0 a moins de restrictions, alors que SSPL-1.0 empêche les fournisseurs cloud de proposer le logiciel comme service managé concurrent.' },
      { q: 'Convoy propose-t-il un cloud managé ?', a: 'Non. Convoy est auto-hébergé uniquement, tu fais tourner et tu maintiens tout toi-même. Hook0 propose un cloud managé (hébergé en Europe) et permet aussi l\'auto-hébergement gratuit avec Docker ou Kubernetes.' },
      { q: 'Comment Hook0 et Convoy se comparent-ils en performances ?', a: 'Hook0 est écrit en Rust, donc pas de pauses garbage collection. Latence plus prévisible et moins de mémoire sous charge. Convoy est écrit en Go, qui tourne bien mais a un overhead GC. Côté infra, tous deux ont besoin de PostgreSQL, mais Convoy demande aussi Redis.' },
      { q: 'Lequel est meilleur pour l\'auto-hébergement ?', a: 'Les deux peuvent être auto-hébergés, mais avec Convoy c\'est ta seule option. Hook0 supporte Docker Compose et Kubernetes pour de l\'auto-hébergement gratuit, et a aussi un cloud managé si tu préfères éviter le boulot ops. Une différence pratique, Hook0 n\'a besoin que de PostgreSQL. Convoy demande PostgreSQL et Redis.' },
    ],
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-alternatives', label: 'Alternatives à Hook0' },
      { enSlug: 'self-hosted-webhooks', label: 'Webhooks auto-hébergés' },
    ],
  },
};
