// Per-page strings for security (FR).
// /humanizer pro + legal-reviewer applied.
// Process claims uniquement (« conçue pour le RGPD »), pas de certification implicite.
// Hébergement = Clever Cloud (France), CDN Cloudflare (USA) divulgué.
// Liens vers /data-processing-addendum et /gdpr-subprocessors gardent leur slug EN
// (pages pas encore traduites).
module.exports = {
  "pageTitle": "Sécurité et conformité chez Hook0 | Hook0",
  "pageDescription": "Pratiques de sécurité Hook0, conçu pour le RGPD, chiffrement TLS, signatures HMAC, sauvegardes chiffrées. Données applicatives hébergées en France chez Clever Cloud, CDN Cloudflare (USA).",
  "pageModified": "2026-06-27",
  "hero": {
    "eyebrow": "Confiance et sécurité",
    "h1": "Sécurité et conformité",
    "subtitle": "Notre approche de la protection de tes données, dans le détail."
  },
  "sections": [
    {
      "h2": "RGPD, conformité et certification",
      "cards": [
        {
          "bodyHtml": "Dès que tu traites des données de l'Union européenne via un prestataire (comme Hook0), tu as besoin d'un cadre contractuel avec chaque prestataire. C'est ce qui permet à l'UE de savoir que tu travailles uniquement avec des entreprises pleinement alignées sur le Règlement général sur la protection des données (RGPD)."
        }
      ]
    },
    {
      "h2": "Accord de traitement des données (DPA)",
      "cards": [
        {
          "bodyHtml": "Un accord de traitement des données (DPA), aussi appelé Data Processing Addendum, est un contrat entre les responsables de traitement et les sous-traitants, ou entre sous-traitants et leurs propres sous-traitants ultérieurs.\n                        <a href=\"/data-processing-addendum\" class=\"text-green-400 hover:text-green-300 transition-colors ml-1\">En savoir plus (en anglais)</a>."
        }
      ]
    },
    {
      "h2": "Sous-traitants ultérieurs",
      "cards": [
        {
          "bodyHtml": "Au sens du RGPD, un sous-traitant ultérieur est toute entreprise ou tout prestataire par lequel les données client peuvent transiter du fait de l'utilisation du service Hook0.\n                        <a href=\"/gdpr-subprocessors\" class=\"text-green-400 hover:text-green-300 transition-colors ml-1\">En savoir plus (en anglais)</a>."
        }
      ]
    },
    {
      "h2": "PCI DSS",
      "cards": [
        {
          "bodyHtml": "Les informations de paiement et de carte bancaire sont prises en charge par <a href=\"https://stripe.com/docs/security\" class=\"text-green-400 hover:text-green-300 transition-colors\">Stripe</a>, audité par un PCI Qualified Security Assessor indépendant et certifié PCI Level 1 Service Provider, le niveau le plus exigeant du secteur des paiements. Hook0 ne reçoit pas de données de carte bancaire en routine, ce qui le rend conforme aux standards PCI DSS dans la majorité des cas d'usage."
        }
      ]
    },
    {
      "h2": "Divulgation de vulnérabilités",
      "cards": [
        {
          "paragraphs": [
            "Pour signaler une vulnérabilité ou tout autre souci de sécurité sur un produit Hook0, écris à <a href=\"mailto:security@hook0.com\" class=\"text-green-400 hover:text-green-300 transition-colors\">security@hook0.com</a>.",
            "Joins une preuve de concept, la liste des outils utilisés (avec leurs versions) et la sortie complète de ces outils. Toutes les divulgations sont prises très au sérieux. Dès réception, chaque vulnérabilité est vérifiée rapidement avant les étapes nécessaires pour la corriger. Une fois validée, des points de statut réguliers sont envoyés au fur et à mesure de la correction.",
            "Pour chiffrer les informations sensibles que tu envoies, la clé PGP est <a href=\"https://keybase.io/fgribreau\" class=\"text-green-400 hover:text-green-300 transition-colors\">disponible sur Keybase</a>.",
            "Un bug bounty ouvert récompense les vulnérabilités critiques signalées sur l'API Hook0 (https://app.hook0.com/api/v1/)."
          ]
        }
      ]
    },
    {
      "h2": "Sécurité de l'infrastructure et du réseau",
      "cards": [
        {
          "h3": "Contrôle d'accès physique",
          "bodyHtml": "Hook0 est hébergé sur la <a href=\"https://www.clever-cloud.com/\" class=\"text-green-400 hover:text-green-300 transition-colors\">plateforme Clever Cloud</a>, en France. Les datacenters Clever Cloud appliquent un modèle de sécurité en couches, avec de larges protections, notamment :",
          "bullets": [
            "Badges électroniques d'accès conçus sur mesure",
            "Alarmes et clôtures périmétriques",
            "Barrières d'accès véhicules et détecteurs de métaux",
            "Authentification biométrique"
          ],
          "footHtml": "Les équipes Hook0 n'ont aucun accès physique aux datacenters Clever Cloud, ni aux serveurs, ni aux équipements réseau, ni au stockage."
        },
        {
          "h3": "Contrôle d'accès logique",
          "bodyHtml": "Hook0 est l'administrateur déclaré de son infrastructure chez Clever Cloud. Seuls les membres autorisés de l'équipe ops Hook0 peuvent configurer l'infrastructure, au besoin, derrière un VPN authentifié à deux facteurs. Des clés privées spécifiques sont exigées par serveur et stockées dans un emplacement chiffré et sécurisé."
        },
        {
          "h3": "Audits indépendants",
          "bodyHtml": "Clever Cloud passe régulièrement divers audits indépendants tiers et fournit la vérification des contrôles de conformité de ses datacenters, de son infrastructure et de ses opérations. Cela inclut, sans s'y limiter, la certification SOC 2 conforme à SSAE 16 et la certification ISO 27001."
        }
      ]
    },
    {
      "h2": "Continuité d'activité et reprise après sinistre",
      "cards": [
        {
          "h3": "Haute disponibilité",
          "bodyHtml": "Chaque brique du service Hook0 repose sur des serveurs correctement dimensionnés et redondés (load balancers, serveurs web, bases de données répliquées) pour faire face aux pannes. La maintenance régulière retire des serveurs de la rotation sans impacter la disponibilité."
        },
        {
          "h3": "Continuité d'activité",
          "bodyHtml": "Hook0 conserve des sauvegardes chiffrées horaires dans plusieurs régions Clever Cloud. Même si ce scénario n'est jamais attendu, en cas de perte de données de production (perte des bases primaires), les données organisationnelles sont restaurées à partir de ces sauvegardes."
        },
        {
          "h3": "Reprise après sinistre",
          "bodyHtml": "En cas de panne touchant une région entière, Hook0 reconstruit un environnement équivalent dans une autre région Clever Cloud. L'équipe ops Hook0 a une expérience solide des migrations de région complètes."
        }
      ]
    },
    {
      "h2": "Sécurité de l'entreprise",
      "cards": [
        {
          "h3": "Protection contre les logiciels malveillants",
          "bodyHtml": "Chez Hook0, les bonnes pratiques de sécurité commencent par notre propre équipe. Nous mettons un point d'honneur à nous protéger contre les menaces internes et les vulnérabilités locales."
        },
        {
          "h3": "Gestion des risques",
          "paragraphs": [
            "Hook0 suit les procédures de gestion des risques décrites dans le <a href=\"http://csrc.nist.gov/publications/PubsSPs.html\" class=\"text-green-400 hover:text-green-300 transition-colors\">NIST SP 800-30</a>, soit neuf étapes d'évaluation des risques et sept étapes de mitigation.",
            "Toute évolution produit Hook0 passe par revue de code, CI et pipeline de build avant d'atteindre les serveurs de production. Seuls les employés désignés de l'équipe ops Hook0 ont un accès SSH aux serveurs de production.",
            "Hook0 réalise des évaluations de risques tout au long du cycle de vie du produit, selon les standards de la <a href=\"https://www.law.cornell.edu/cfr/text/45/164.308\" class=\"text-green-400 hover:text-green-300 transition-colors\">HIPAA Security Rule, 45 CFR 164.308</a>."
          ]
        },
        {
          "h3": "Politiques de sécurité et formation",
          "bodyHtml": "Hook0 maintient un wiki interne des politiques de sécurité, mis à jour en continu et revu chaque année pour combler les écarts. Tous les nouveaux employés reçoivent un onboarding et une formation aux systèmes, avec revue des politiques de sécurité."
        },
        {
          "h3": "Politique de divulgation",
          "paragraphs": [
            "Hook0 suit le processus de gestion d'incidents recommandé par le <a href=\"https://www.sans.org/reading-room/whitepapers/incident/incident-handlers-handbook-33901\" class=\"text-green-400 hover:text-green-300 transition-colors\">SANS</a>, qui inclut l'identification, le confinement, l'éradication, la récupération, la communication et la documentation des événements de sécurité.",
            "Hook0 publie en direct l'état opérationnel et les incidents sur sa <a href=\"https://status.hook0.com/\" class=\"text-green-400 hover:text-green-300 transition-colors\">status page</a>. Tout incident connu y est signalé, ainsi que sur le <a href=\"https://twitter.com/hook0_\" class=\"text-green-400 hover:text-green-300 transition-colors\">flux Twitter</a>."
          ]
        }
      ]
    }
  ]
};
