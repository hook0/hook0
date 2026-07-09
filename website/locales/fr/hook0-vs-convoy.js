// Per-page strings for hook0-vs-convoy (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon.
// SSPL pour Hook0 = « code source ouvert (SSPL-1.0) ». Convoy = Elastic License v2.0 (non OSI),
// donc « code source disponible », jamais « open source » nu.
// Faits rafraîchis 2026-07-08 (snapshot concurrent) : Convoy ACTIF (v26.6.2 du 08/07/2026),
// cloud sans pricing public ni résidence UE managée, tarifs 0 $ -> 999 $/mois flat.
module.exports = {
  pageTitle: 'Hook0 vs Convoy : plateformes webhook comparées | Hook0',
  pageDescription: 'Compare Hook0 (Rust, SSPL-1.0, cloud hébergé en UE dès 59 €/mois) et Convoy (Go, Elastic License v2.0, de 0 à 999 $/mois). Features, licences et prix côte à côte.',
  pageModified: '2026-07-08',
  breadcrumb: 'Hook0 vs Convoy',
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Hook0 vs Convoy',
    titleAccent: 'Même problème, compromis différents',
    subtitle: 'Tous deux publient leur code source intégral. Tous deux sur PostgreSQL. Les vraies différences sont ailleurs, Rust vs Go, SSPL-1.0 vs Elastic License v2.0, et une grille tarifaire progressive face à un forfait à 999 $ par mois. Cette page décortique ce qui compte vraiment quand tu choisis pour la prod.',
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
  },
  differentiators: {
    eyebrow: 'Pourquoi Hook0',
    h2: 'Différences clés',
    cards: [
      { title: 'Un tarif intermédiaire vs un saut de 0 à 999 $', body: 'La grille payante de Convoy n\'a qu\'une marche, le tier Community est gratuit, puis Premium arrive à 999 $/mois flat (transformations JS, RBAC et white-label inclus). Rien entre les deux. Hook0 Cloud démarre gratuit, puis passe à Startup à 59 €/mois et Pro à 190 €/mois, une équipe qui grandit n\'affronte jamais une falaise tarifaire sans palier intermédiaire.' },
      { title: 'Cloud managé UE vs auto-hébergement pour la résidence des données', body: 'Le cloud managé de Hook0 fait tourner son plan de données applicatif en France chez Clever Cloud (CDN Cloudflare US divulgué dans notre <a href="/fr/accord-traitement-donnees" class="underline">DPA</a>), conçu pour la conformité RGPD. Convoy a aussi une offre cloud, mais aucune option de résidence UE managée, choisir la région où vivent tes données webhook impose l\'auto-hébergement, donc le monitoring, les backups, le scaling et l\'uptime sont pour toi.' },
      { title: 'Rust vs Go', body: 'Hook0 est écrit en Rust. Pas de garbage collector, donc pas de pauses GC, moins de mémoire utilisée et une latence plus prévisible sous charge. Convoy est écrit en Go, débit correct mais avec garbage collection. À haut volume, l\'écart se voit sur les latences en queue.' },
      { title: 'SSPL-1.0 vs Elastic License v2.0', body: 'Convoy utilise la licence Elastic v2.0, code source intégral disponible, mais proposer Convoy en service managé exige un accord commercial. Hook0 utilise SSPL-1.0, code source intégral disponible, mais les fournisseurs cloud ne peuvent pas le revendre comme service concurrent. Les deux sont des licences à code source disponible et aucune n\'est approuvée par l\'OSI. La différence pratique porte sur l\'activité restreinte, pas sur la quantité de code que tu peux lire.' },
    ],
  },
  comparison: {
    eyebrow: 'Comparaison de fonctionnalités',
    h2: 'Côte à côte',
    headers: { feature: 'Fonctionnalité', hook0: 'Hook0', convoy: 'Convoy' },
    rows: [
      { feature: 'Licence', hook0Html: 'SSPL-1.0 (source intégrale disponible, non approuvée OSI)', convoyHtml: 'Elastic License v2.0 (source intégrale disponible, non approuvée OSI)' },
      { feature: 'Langage', hook0Html: 'Rust', convoyHtml: 'Go' },
      { feature: 'Base de données', hook0Html: 'PostgreSQL seulement', convoyHtml: 'PostgreSQL + Redis' },
      { feature: 'Sens des webhooks', hook0Html: 'Sortants (envoi)', convoyHtml: 'Sortants + entrants' },
      { feature: 'Cloud managé', hook0Html: 'Oui (Clever Cloud FR, CDN Cloudflare US divulgué)', convoyHtml: 'Oui (pas de pricing public, pas de résidence UE managée)' },
      { feature: 'Auto-hébergement', hook0Html: 'Gratuit (Docker / K8s)', convoyHtml: 'Gratuit (tier Community)' },
      { feature: 'Plans payants', hook0Html: 'Startup 59 €/mois, Pro 190 €/mois', convoyHtml: 'Premium 999 $/mois (flat), Enterprise sur devis' },
      { feature: 'SOC 2', hook0Html: 'Prévu', convoyHtml: 'SOC 2 Type 1' },
      { feature: 'Signatures HMAC', hook0Html: 'Oui', convoyHtml: 'Oui' },
      { feature: 'Logique de relances', hook0Html: 'Configurable 2-phases (rapide + lent, defaults intelligents)', convoyHtml: 'Configurable' },
      { feature: 'Dépôt principal', hook0Html: '<a href="https://github.com/hook0/hook0" class="underline">GitHub</a> + <a href="https://gitlab.com/hook0/hook0" class="underline">GitLab</a>', convoyHtml: '<a href="https://github.com/frain-dev/convoy" class="underline">GitHub</a> (~2,8k stars)' },
      { feature: 'Financement', hook0Html: '100% bootstrappé', convoyHtml: 'VC-backed (YC W22, Frain Technologies)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    items: [
      { q: 'Convoy est-il open source ?', a: 'Convoy publie son code source intégral sous licence Elastic v2.0, qui n\'est pas une licence open source approuvée par l\'OSI, elle interdit de proposer Convoy en service managé sans accord commercial. Hook0 est dans la même famille, code source intégral sous SSPL-1.0, non approuvée par l\'OSI elle aussi, avec une restriction visant les fournisseurs cloud qui le revendraient. Si ta politique d\'achat exige strictement une licence OSI, aucun des deux ne qualifie.' },
      { q: 'Convoy propose-t-il un cloud managé ?', a: 'Oui. Convoy propose une version cloud (le trial donne 1 projet et 100 événements par jour) mais ne publie pas de pricing cloud, et il n\'y a pas d\'option de résidence UE managée, choisir où vivent tes données webhook impose l\'auto-hébergement. Le cloud managé de Hook0 est hébergé en UE dès le tier gratuit, avec des plans payants à 59 € et 190 € par mois.' },
      { q: 'Comment Hook0 et Convoy se comparent-ils sur les prix ?', a: 'En auto-hébergement, les deux sont gratuits. Pour les fonctions payantes, Convoy passe directement du tier Community gratuit à Premium à 999 $/mois flat, rien entre les deux. Hook0 Cloud a un tier gratuit, puis Startup à 59 €/mois et Pro à 190 €/mois. Si une facture flat tout inclus convient à ton équipe, le Premium de Convoy est prévisible. Si tu veux démarrer petit et monter en charge, Hook0 couvre le milieu de marché que Convoy saute.' },
      { q: 'Comment Hook0 et Convoy se comparent-ils en performances ?', a: 'Hook0 est écrit en Rust, donc pas de pauses garbage collection. Latence plus prévisible et moins de mémoire sous charge. Convoy est écrit en Go, qui tourne bien mais a un overhead GC. Côté infra, tous deux ont besoin de PostgreSQL, mais Convoy demande aussi Redis.' },
      { q: 'Que fait Convoy mieux que Hook0 ?', a: 'Convoy gère les webhooks entrants et sortants dans un seul produit, alors que Hook0 se concentre sur la livraison sortante. Convoy a aussi une attestation SOC 2 Type 1, plus d\'étoiles GitHub (~2 800), des clients fintech de référence comme Xendit et PiggyVest, et un tier Premium flat à 999 $/mois que certaines équipes préfèrent pour la prévisibilité de facturation.' },
      { q: 'Convoy est-il toujours maintenu ?', a: 'Oui. Convoy publie 2 à 3 releases par mois (la v26.6.2 est sortie en juillet 2026), son blog est actif et le dépôt GitHub compte plusieurs contributeurs réguliers. Toute affirmation « Convoy est mort » trouvée en ligne est périmée.' },
    ],
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-alternatives', label: 'Alternatives à Hook0' },
      { enSlug: 'self-hosted-webhooks', label: 'Webhooks auto-hébergés' },
      { enSlug: 'webhook-cost-comparison', label: 'Comparatif de coût webhook' },
      { enSlug: 'eu-webhook-infrastructure', label: 'Infrastructure webhook européenne' },
    ],
  },
};
