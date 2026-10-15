// Per-page strings for hook0-vs-convoy (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon.
// SSPL pour Hook0 = « open source (SSPL-1.0) ». Convoy = Elastic License v2.0 (non OSI),
// donc « code source disponible », jamais « open source » nu.
// Faits rafraîchis 2026-09-29 (snapshot concurrent) : Convoy ACTIF (v26.8.0 du 28/09/2026,
// commits quotidiens, ~2 875 stars). Tarifs relus le 2026-10-15 : Convoy Cloud Pro 99 $/mois flat
// (25 events/s), Premium 499 $/mois, région US ou UE au signup ; Premium auto-hébergé 999 $/mois.
module.exports = {
  pageTitle: 'Hook0 vs Convoy : plateformes webhook comparées | Hook0',
  pageDescription: 'Compare Hook0 (Rust, SSPL-1.0, cloud hébergé en UE dès 59 €/mois) et Convoy (Go, Elastic License v2.0, cloud dès 99 $/mois). Features, licences et prix côte à côte.',
  pageModified: '2026-10-15',
  breadcrumb: 'Hook0 vs Convoy',
  tldr: {
    h2: 'En bref',
    body: "Hook0 et Convoy publient tous deux leur code source intégral et tournent sur PostgreSQL, donc les vrais compromis sont ailleurs. Hook0 est écrit en Rust sous SSPL-1.0, avec un cloud managé en UE à partir de 59 €/mois et un palier Pro à 190 €. Convoy est écrit en Go sous Elastic License v2.0. Son cloud propose un essai de 14 jours, puis Pro à 99 $/mois forfaitaires pour 25 events par seconde et Premium à 499 $/mois, en région US ou UE ; sa licence Premium auto-hébergée coûte 999 $/mois. Choisis Convoy pour l'inbound et l'outbound dans un seul produit et un forfait au débit ; choisis Hook0 pour un moteur en Rust, un tier gratuit permanent, un opérateur français et un serveur auto-hébergé avec toutes ses fonctions.",
  },
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Hook0 vs Convoy',
    titleAccent: 'Même problème, compromis différents',
    subtitle: 'Tous deux publient leur code source intégral. Tous deux sur PostgreSQL. Les vraies différences sont ailleurs, Rust vs Go, SSPL-1.0 vs Elastic License v2.0, et des quotas d\'events par jour face à des forfaits au débit. Cette page décortique ce qui compte vraiment quand tu choisis pour la prod.',
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
  },
  differentiators: {
    eyebrow: 'Pourquoi Hook0',
    h2: 'Différences clés',
    cards: [
      { title: 'Quotas d\'events vs forfaits au débit', body: 'Convoy facture son cloud au débit : Pro coûte 99 $/mois forfaitaires pour 25 events par seconde, Premium 499 $/mois avec des limites sur mesure, et rien ne reste gratuit après l\'essai de 14 jours. En auto-hébergé, ses fonctions Premium (transformations JS, RBAC, white-label) coûtent 999 $/mois. Hook0 Cloud facture au volume quotidien : gratuit jusqu\'à 100 events par jour, Startup à 59 €/mois, Pro à 190 €/mois pour 100 000 events par jour, et le dépassement est facturé à l\'event au lieu de bloquer la livraison. Le serveur Hook0 auto-hébergé inclut toutes les fonctions sans coût de licence.' },
      { title: 'Une région UE vs un opérateur français', body: 'Le cloud managé de Hook0 fait tourner son plan de données applicatif en France chez Clever Cloud (CDN Cloudflare US divulgué dans notre <a href="/fr/accord-traitement-donnees" class="underline">DPA</a>), conçu pour la conformité RGPD, et Hook0 est exploité par une société française. Convoy Cloud te laisse choisir une région US ou UE à l\'inscription et liste des options de résidence des données sur son offre Enterprise ; l\'adresse de la société Convoy est à San Francisco. Où sont tes données et quel droit s\'applique au fournisseur sont deux questions distinctes.' },
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
      { feature: 'Cloud managé', hook0Html: 'Oui (Clever Cloud FR, CDN Cloudflare US divulgué)', convoyHtml: 'Oui (région US ou UE, choisie à l\'inscription)' },
      { feature: 'Auto-hébergement', hook0Html: 'Gratuit (Docker / K8s)', convoyHtml: 'Gratuit (tier Community)' },
      { feature: 'Plans payants', hook0Html: 'Startup 59 €/mois, Pro 190 €/mois', convoyHtml: 'Cloud Pro 99 $/mois, Cloud Premium 499 $/mois, Premium auto-hébergé 999 $/mois, Enterprise sur devis' },
      { feature: 'SOC 2', hook0Html: 'Prévu', convoyHtml: 'SOC 2 Type 1' },
      { feature: 'Signatures HMAC', hook0Html: 'Oui', convoyHtml: 'Oui' },
      { feature: 'Logique de relances', hook0Html: 'Configurable 2-phases (rapide + lent, defaults intelligents)', convoyHtml: 'Configurable' },
      { feature: 'Dépôt principal', hook0Html: '<a href="https://github.com/hook0/hook0" class="underline">GitHub</a> + <a href="https://gitlab.com/hook0/hook0" class="underline">GitLab</a>', convoyHtml: '<a href="https://github.com/frain-dev/convoy" class="underline">GitHub</a> (~2,9k stars)' },
      { feature: 'Financement', hook0Html: '100% bootstrappé', convoyHtml: 'VC-backed (YC W22, Frain Technologies)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    items: [
      { q: 'Convoy est-il open source ?', a: 'Convoy publie son code source intégral sous licence Elastic v2.0, qui n\'est pas une licence open source approuvée par l\'OSI, elle interdit de proposer Convoy en service managé sans accord commercial. Hook0 est dans la même famille, code source intégral sous SSPL-1.0, non approuvée par l\'OSI elle aussi, avec une restriction visant les fournisseurs cloud qui le revendraient. Si ta politique d\'achat exige strictement une licence OSI, aucun des deux ne qualifie.' },
      { q: 'Convoy propose-t-il un cloud managé ?', a: 'Oui. Convoy Cloud propose un essai de 14 jours (1 projet, 100 événements par jour), puis Pro à 99 $/mois forfaitaires pour 25 events par seconde et Premium à 499 $/mois avec des limites sur mesure. Tu choisis une région US ou UE à l\'inscription. Le cloud managé de Hook0 est hébergé en UE dès le tier gratuit, avec des plans payants à 59 € et 190 € par mois.' },
      { q: 'Comment Hook0 et Convoy se comparent-ils sur les prix ?', a: 'En auto-hébergement, les deux serveurs sont gratuits, mais les fonctions Premium de Convoy coûtent 999 $/mois alors que le serveur Hook0 les inclut toutes. Dans le cloud, Convoy facture un forfait par palier de débit : 99 $/mois pour 25 events par seconde, 499 $/mois pour des limites sur mesure. Hook0 Cloud est gratuit jusqu\'à 100 events par jour, puis 59 €/mois pour 30 000 events par jour et 190 €/mois pour 100 000, avec un dépassement facturé à l\'event. À volume élevé et régulier, Convoy Pro peut coûter moins cher ; pour un volume petit ou en croissance, et pour un tier gratuit qui n\'expire pas, Hook0 coûte moins cher.' },
      { q: 'Comment Hook0 et Convoy se comparent-ils en performances ?', a: 'Hook0 est écrit en Rust, donc pas de pauses garbage collection. Latence plus prévisible et moins de mémoire sous charge. Convoy est écrit en Go, qui tourne bien mais a un overhead GC. Côté infra, tous deux ont besoin de PostgreSQL, mais Convoy demande aussi Redis.' },
      { q: 'Que fait Convoy mieux que Hook0 ?', a: 'Convoy gère les webhooks entrants et sortants dans un seul produit, alors que Hook0 se concentre sur la livraison sortante. Convoy a aussi une attestation SOC 2 Type 1, plus d\'étoiles GitHub (~2 875), des clients fintech de référence cités publiquement, et des forfaits cloud au débit dès 99 $/mois que certaines équipes préfèrent pour la prévisibilité de facturation.' },
      { q: 'Convoy est-il toujours maintenu ?', a: 'Oui. Convoy publie 2 à 3 releases par mois (la v26.8.0 est sortie en septembre 2026), son blog est actif et le dépôt GitHub compte plusieurs contributeurs réguliers. Toute affirmation « Convoy est mort » trouvée en ligne est périmée.' },
    ],
  },
  comparisonSources: {
    competitors: ["Convoy"],
    scope: "Cette page compare les offres publiques, la licence et l'état de maintenance de Hook0 et de Convoy. Les prix sont ceux publiés par chaque éditeur, et la maintenance est tirée de l'activité publique de Convoy sur GitHub.",
    sources: [
      { label: "Releases de Convoy sur GitHub", url: "https://github.com/frain-dev/convoy/releases", consulted: "2026-09-29" },
      { label: "Tarifs Convoy", url: "https://www.getconvoy.io/pricing", consulted: "2026-10-15", archive: "https://web.archive.org/web/20260604143101/https://www.getconvoy.io/pricing" },
      { label: "Licence de Convoy sur GitHub", url: "https://github.com/frain-dev/convoy/blob/main/LICENSE", consulted: "2026-10-04" },
    ],
    updated: "2026-10-15",
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
      { enSlug: 'sovereign-webhooks', label: 'Webhooks souverains' },
    ],
  },
};
