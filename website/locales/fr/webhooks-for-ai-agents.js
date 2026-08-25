// Per-page strings pour webhooks-for-ai-agents (FR, slug : serveur-mcp-webhook).
// Choix du slug : « serveur mcp » est le terme français réellement tapé
// (Google Suggest FR remonte « serveur mcp c'est quoi », « serveur mcp claude »,
// « serveur mcp open source », « serveur mcp gratuit »), tandis que
// « webhook ia » ne remonte rien — les devs francophones gardent le mot
// « webhook » en anglais. D'où « serveur-mcp-webhook » plutôt qu'une
// traduction littérale de « webhooks for ai agents ».
// /humanizer pro + contraintes légales appliquées (cf. website/CLAUDE.local.md) :
//   - Data plane = Clever Cloud SAS (France, EEE). CDN Cloudflare, Inc. (USA)
//     DIVULGUÉ, encadré par CCT 2021 + TIA et, le cas échéant, le DPF UE-USA.
//   - JAMAIS « 100 % souverain », « aucune donnée ne quitte l'UE », « hors CLOUD Act ».
//   - RGPD = claim de processus (« conçu pour »), jamais « certifié ».
//   - Licence = « code source ouvert (SSPL-1.0) », jamais « open source » nu.
//   - Ni SOC 2 ni ISO : Hook0 n'a aucune des deux.
// Registre tu, developer-first. Gardés en anglais : Webhook, Endpoint, Payload,
// Event, HMAC, Dashboard. Traduits : Signature, Livraison, Relances.
// Capacités reprises de documentation/reference/mcp.md (9 outils de lecture,
// 8 d'écriture, 17 au total) — rien au-delà. Aucun claim sur ce que les
// concurrents ne font pas.
// Le texte de faq.items[].a DOIT être identique au corps de carte visible ;
// le JSON-LD FAQPage est généré depuis ce même tableau. Texte brut, pas de HTML.
module.exports = {
  "pageTitle": "Serveur MCP webhook pour agents IA | Hook0",
  "pageDescription": "Le serveur MCP de Hook0 donne à Claude, Cursor et Windsurf 17 outils pour piloter les webhooks que ton produit envoie : créer des souscriptions, émettre des events, rejouer les livraisons en échec.",
  "pageModified": "2026-08-25",
  "track": "fr-mcp-webhooks",
  "hero": {
    "eyebrow": "Serveur MCP webhook",
    "titleLine1": "Ton agent sait lire les webhooks.",
    "titleLine2": "Il sait aussi les envoyer.",
    "subtitle": "Brancher un assistant sur des webhooks, c'est en général consommer les events émis par d'autres. Hook0 prend le problème par l'autre bout : un serveur MCP qui laisse Claude, Cursor ou Windsurf piloter les webhooks que ton propre produit envoie. Déclarer des types d'event, créer des souscriptions, émettre, rejouer ce qui a échoué.",
    "ctaPrimary": "Démarrer gratuitement",
    "ctaSecondary": "Lire la doc MCP",
    "ctaSecondaryHref": "https://documentation.hook0.com/reference/mcp",
    "microcopy": "100 events/jour gratuits. Sans carte bancaire. Le serveur MCP tourne sur ta machine."
  },
  "answer": {
    "h2": "En résumé",
    "lead": "Le serveur MCP de Hook0 est un process local qui expose ton compte Hook0 à un assistant compatible MCP sous forme de 17 outils. Neuf lisent tes applications, types d'event, souscriptions, events et tentatives de livraison. Huit écrivent : déclarer un type d'event, brancher une souscription, émettre un event, rejouer une livraison en échec.",
    "factsLabel": "L'essentiel",
    "facts": [
      { "k": "Installation", "v": "<code class=\"text-green-400\">cargo install hook0-mcp</code>" },
      { "k": "Outils", "v": "17 (9 en lecture, 8 en écriture)" },
      { "k": "Clients", "v": "Claude Desktop, Cursor, Windsurf, Cline, et tout client MCP" },
      { "k": "Transport", "v": "stdio par défaut, SSE si tu le fais tourner en service" },
      { "k": "Cible", "v": "Hook0 Cloud, ou ton instance via <code class=\"text-green-400\">HOOK0_API_URL</code>" },
      { "k": "Pour le brider", "v": "<code class=\"text-green-400\">HOOK0_READ_ONLY=true</code> et un service token atténué" },
      { "k": "Licence", "v": "Code source ouvert (SSPL-1.0), sans rétention open-core" }
    ]
  },
  "socialProof": true,
  "sides": {
    "eyebrow": "Deux directions",
    "h2": "Webhooks et agents pointent dans deux sens différents",
    "subtitle": "On en parle souvent comme d'un seul sujet. Ce n'est pas le même problème, et ça ne se règle pas avec le même outil.",
    "cards": [
      {
        "title": "Un agent qui reçoit des webhooks",
        "bodyHtml": "Quelque chose se passe dans Stripe, GitHub ou un CRM, et un agent doit réagir. Le travail est entrant : un endpoint public ou un tunnel, la vérification de signature, la déduplication, et ne pas relancer deux fois le même agent. L'outillage de ce côté-là est bien couvert."
      },
      {
        "title": "Un agent qui pilote ce que tu envoies",
        "bodyHtml": "Cette fois c'est toi qui émets. Tes clients souscrivent à <code class=\"text-green-400\">order.completed</code> et comptent le recevoir. Le travail est sortant : déclarer les types d'event, brancher les souscriptions, comprendre pourquoi une tentative a échoué, la rejouer une fois le récepteur réparé. C'est ce côté-là que le serveur MCP de Hook0 expose à ton assistant."
      }
    ],
    "note": "Si tu édites un produit auquel d'autres systèmes souscrivent, c'est la deuxième direction qui te coûte des tickets de support."
  },
  "versus": {
    "eyebrow": "MCP ou webhook",
    "h2": "Deux protocoles, deux étages",
    "intro": "On les compare parce que les deux transportent des events. Ils ne travaillent pas au même étage, et une installation qui tient la route utilise les deux.",
    "rows": [
      {
        "label": "Un webhook",
        "bodyHtml": "Une requête HTTP que ton produit envoie vers une URL déclarée par un client, quand quelque chose se passe chez toi. Unidirectionnelle, de machine à machine, signée pour que le récepteur puisse s'y fier, relancée quand il est indisponible. C'est le plan de données : il déplace l'event."
      },
      {
        "label": "MCP",
        "bodyHtml": "Le Model Context Protocol, la façon dont un assistant découvre et appelle des outils. Requête/réponse, conduit par une personne dans une conversation, exécuté en local. C'est un plan de contrôle : il pilote ce qui déplace l'event."
      },
      {
        "label": "Les deux ensemble",
        "bodyHtml": "Ton produit continue d'émettre ses events via l'API REST ou un SDK, sans rien changer. MCP sert quand tu dois déclarer un nouveau type d'event, réorienter une souscription, ou comprendre pourquoi la livraison d'hier a échoué, sans ouvrir le dashboard."
      }
    ]
  },
  "tools": {
    "eyebrow": "Serveur MCP",
    "h2": "Ce que l'assistant peut vraiment faire",
    "subtitle": "Dix-sept outils, répartis entre lecture et écriture. C'est la liste livrée, pas une roadmap. Chacun est documenté, avec son prompt d'exemple, dans la référence MCP.",
    "headers": {
      "tool": "Outil",
      "does": "Ce qu'il fait",
      "say": "Ce que tu tapes"
    },
    "groups": [
      {
        "label": "Lecture (9 outils)",
        "rows": [
          { "tool": "list_organizations", "does": "Lister les organisations auxquelles tu as accès", "say": "&laquo;&nbsp;Montre mes organisations&nbsp;&raquo;" },
          { "tool": "list_applications", "does": "Lister les applications d'une organisation", "say": "&laquo;&nbsp;Quelles applis j'ai&nbsp;?&nbsp;&raquo;" },
          { "tool": "get_application", "does": "Lire le détail d'une application", "say": "&laquo;&nbsp;Détaille l'appli X&nbsp;&raquo;" },
          { "tool": "list_event_types", "does": "Lister les types d'event déclarés sur une application", "say": "&laquo;&nbsp;Quels types d'event sont déclarés&nbsp;?&nbsp;&raquo;" },
          { "tool": "list_subscriptions", "does": "Lister les souscriptions webhook et leur configuration", "say": "&laquo;&nbsp;Montre tous mes webhooks&nbsp;&raquo;" },
          { "tool": "get_subscription", "does": "Lire la configuration d'une souscription", "say": "&laquo;&nbsp;Montre la config du webhook&hellip;&nbsp;&raquo;" },
          { "tool": "list_events", "does": "Lister les events émis par une application", "say": "&laquo;&nbsp;Montre les events récents&nbsp;&raquo;" },
          { "tool": "get_event", "does": "Lire un event, payload compris", "say": "&laquo;&nbsp;Montre l'event abc123&nbsp;&raquo;" },
          { "tool": "list_request_attempts", "does": "Lister les tentatives de livraison d'un event", "say": "&laquo;&nbsp;Montre l'historique de livraison de l'event X&nbsp;&raquo;" }
        ]
      },
      {
        "label": "Écriture (8 outils)",
        "highlight": true,
        "rows": [
          { "tool": "create_application", "does": "Créer une application", "say": "&laquo;&nbsp;Crée une appli Order Service&nbsp;&raquo;" },
          { "tool": "delete_application", "does": "Supprimer une application", "say": "&laquo;&nbsp;Supprime l'appli de test&nbsp;&raquo;" },
          { "tool": "create_event_type", "does": "Déclarer un nouveau type d'event", "say": "&laquo;&nbsp;Ajoute le type order.completed&nbsp;&raquo;" },
          { "tool": "create_subscription", "does": "Créer une souscription webhook vers une URL", "say": "&laquo;&nbsp;Crée un webhook vers https://&hellip;&nbsp;&raquo;" },
          { "tool": "update_subscription", "does": "Modifier ou désactiver une souscription existante", "say": "&laquo;&nbsp;Désactive le webhook de&hellip;&nbsp;&raquo;" },
          { "tool": "delete_subscription", "does": "Supprimer une souscription", "say": "&laquo;&nbsp;Retire le webhook de staging&nbsp;&raquo;" },
          { "tool": "ingest_event", "does": "Émettre un event, le côté envoi, depuis l'assistant", "say": "&laquo;&nbsp;Envoie un event user.created de test&nbsp;&raquo;" },
          { "tool": "retry_delivery", "does": "Rejouer une livraison en échec", "say": "&laquo;&nbsp;Rejoue la livraison échouée de l'event X&nbsp;&raquo;" }
        ]
      }
    ],
    "footnote": "S'y ajoutent huit URI de ressources sous hook0:// pour les accès directs, et trois prompts guidés pour les enchaînements qu'on répète : créer une souscription, déboguer une livraison, monter une application.",
    "footHref": "https://documentation.hook0.com/reference/mcp",
    "footLabel": "Référence complète des outils, avec la configuration pour Claude Desktop, Cursor, Windsurf et Cline"
  },
  "setup": {
    "eyebrow": "Mise en route",
    "h2": "Trois étapes avant ton premier prompt",
    "intro": "Le serveur est un binaire Rust que tu installes une fois. Rien de nouveau ne tourne côté Hook0.",
    "steps": [
      {
        "n": "1",
        "title": "Installer le serveur",
        "bodyHtml": "Il est publié sur crates.io et compile en un seul binaire.",
        "code": "cargo install hook0-mcp"
      },
      {
        "n": "2",
        "title": "Créer un service token",
        "bodyHtml": "Dans le dashboard Hook0, section service tokens de ton organisation. Atténue-le aux applications que l'assistant doit atteindre avant de le coller où que ce soit.",
        "code": ""
      },
      {
        "n": "3",
        "title": "Le déclarer dans ton assistant",
        "bodyHtml": "Claude Desktop lit <code class=\"text-green-400\">claude_desktop_config.json</code>. Cursor, Windsurf et Cline prennent le même bloc dans leur propre fichier de configuration.",
        "code": "{\n  \"mcpServers\": {\n    \"hook0\": {\n      \"command\": \"hook0-mcp\",\n      \"env\": {\n        \"HOOK0_API_TOKEN\": \"ton-service-token-ici\"\n      }\n    }\n  }\n}"
      }
    ],
    "outro": "Redémarre l'assistant et demande-lui ce que tu irais autrement chercher à la souris : &laquo;&nbsp;pourquoi ma dernière livraison de webhook a échoué&nbsp;?&nbsp;&raquo;",
    "docsHref": "https://documentation.hook0.com/reference/mcp",
    "docsLabel": "Chemins des fichiers de configuration par assistant, variables d'environnement et mode SSE"
  },
  "guardrails": {
    "eyebrow": "Garde-fous",
    "h2": "Confier ton infrastructure de livraison à un agent",
    "intro": "L'accès en écriture à des webhooks de production ne se donne pas à la légère. Trois contrôles sont livrés avec le serveur, et ils se combinent.",
    "cards": [
      {
        "title": "Mode lecture seule",
        "bodyHtml": "Mets <code class=\"text-green-400\">HOOK0_READ_ONLY=true</code> et le serveur n'annonce que les neuf outils de lecture. L'assistant peut enquêter sur une livraison en échec sans rien pouvoir modifier au passage."
      },
      {
        "title": "Tokens atténués",
        "bodyHtml": "Le mode lecture seule restreint la liste d'outils ; le token, lui, garde les droits qu'on lui a donnés. L'atténuation limite un token à des applications précises et peut porter une expiration, appliquée côté API et non côté client. Utilise les deux : ils couvrent des défaillances différentes."
      },
      {
        "title": "Il tourne là où tu le lances",
        "bodyHtml": "Le serveur MCP est un process local qui parle à l'API Hook0. L'assistant voit ce que retournent les outils qu'il appelle, et rien d'autre n'est transmis ailleurs. Pointe <code class=\"text-green-400\">HOOK0_API_URL</code> vers ton instance et les mêmes outils pilotent un déploiement auto-hébergé."
      }
    ]
  },
  "platform": {
    "eyebrow": "En dessous",
    "h2": "L'agent est une interface, pas la garantie",
    "intro": "Le langage naturel change la façon dont tu pilotes la livraison des webhooks. Il ne livre rien par lui-même. Voilà ce qui livre.",
    "cards": [
      {
        "title": "Signé, relancé, journalisé",
        "bodyHtml": "Chaque tentative porte une signature HMAC-SHA256. Les relances sont en deux phases et configurables : un souscripteur indisponible pendant une heure ne te coûte pas cette heure d'events. Chaque tentative est journalisée, ce qui transforme le &laquo;&nbsp;pourquoi ça a échoué&nbsp;&raquo; en consultation plutôt qu'en supposition."
      },
      {
        "title": "Un data plane européen, sur toutes les offres",
        "bodyHtml": "Payloads, base de données et sauvegardes tournent sur l'infrastructure de Clever Cloud SAS en France, dans l'EEE, y compris sur l'offre gratuite. Le CDN devant est Cloudflare, Inc. (USA), divulgué dans la <a href=\"/fr/sous-traitants-rgpd\" class=\"text-green-400 hover:text-green-300 transition-colors\">liste publique des sous-traitants</a> avec son mécanisme de transfert. Le détail sur <a href=\"/fr/infrastructure-webhook-europeenne\" class=\"text-green-400 hover:text-green-300 transition-colors\">l'infrastructure webhook européenne</a>."
      },
      {
        "title": "Un code que tu peux emporter",
        "bodyHtml": "Hook0 est à code source ouvert (SSPL-1.0), sans rétention open-core : le service hébergé fait tourner le code que tu peux faire tourner. Le serveur MCP, l'API et le portail souscripteur se comportent pareil face à une instance auto-hébergée."
      }
    ]
  },
  "faq": {
    "eyebrow": "Questions",
    "h2": "Avant de pointer un assistant sur ta production",
    "items": [
      {
        "q": "Comment utiliser des webhooks avec des outils MCP ?",
        "a": "Installe hook0-mcp, crée un service token Hook0, et déclare le serveur dans le fichier de configuration de ton assistant. À partir de là, il dispose de dix-sept outils sur ton compte : lister l'existant, déclarer un type d'event, créer une souscription vers une URL, émettre un event de test, rejouer une livraison en échec. Ton application, elle, continue d'émettre ses vrais events via l'API REST ou un SDK, et ce chemin ne bouge pas."
      },
      {
        "q": "Quelle est la différence entre MCP et un webhook ?",
        "a": "Un webhook est une requête HTTP que ton produit envoie vers une URL déclarée par un client, unidirectionnelle et de machine à machine, signée pour que le récepteur puisse s'y fier. MCP est la façon dont un assistant découvre et appelle des outils, en requête/réponse, conduit par une personne dans une conversation. Le webhook déplace l'event ; MCP pilote le système qui le déplace. La plupart des installations qui utilisent les deux ne touchent pas au chemin webhook et réservent MCP au travail d'exploitation."
      },
      {
        "q": "L'assistant voit-il le contenu de mes events ?",
        "a": "Il voit ce que retournent les outils qu'il appelle, et lire un event retourne son payload. Rien n'est transmis à un tiers par Hook0 : le serveur MCP est un process local qui parle directement à l'API Hook0. Si les payloads doivent rester hors de la conversation, atténue le token aux applications qui ne portent pas de données sensibles."
      },
      {
        "q": "Quels assistants fonctionnent avec ?",
        "a": "Claude Desktop, Cursor, Windsurf et Cline sont documentés avec leur fichier de configuration, et tout client compatible MCP marche de la même façon. ChatGPT ne supporte pas MCP nativement à ce jour."
      },
      {
        "q": "Qu'est-ce qui empêche un agent de supprimer une souscription de production ?",
        "a": "Deux choses, et elles se combinent utilement. Le mode lecture seule retire les outils d'écriture de la liste que l'assistant peut voir. L'atténuation de token restreint ce que le token lui-même peut toucher, appliquée côté API, de sorte qu'une erreur du client ne peut pas la dépasser. Les ressources supprimées ne sont pas restaurables automatiquement : c'est la raison de mettre les deux avant de pointer un assistant sur la production."
      },
      {
        "q": "Que deviennent les events quand l'endpoint d'un souscripteur est indisponible un moment ?",
        "a": "Hook0 relance la livraison sur un calendrier en deux phases, configurable, plutôt que de l'abandonner au premier échec, et enregistre chaque tentative avec sa réponse. Une fois le récepteur réparé, tu rejoues ce qui a échoué, depuis le dashboard, depuis l'API, ou en demandant à l'assistant de relancer cette livraison. L'event n'est pas perdu pendant que l'endpoint est injoignable."
      },
      {
        "q": "Comment le récepteur vérifie-t-il un payload envoyé par Hook0 ?",
        "a": "Chaque tentative porte une signature HMAC-SHA256 calculée depuis le payload et le secret de la souscription. Le récepteur la recalcule et la compare avant d'agir sur l'event, ce qui empêche une requête forgée de déclencher un traitement. Le schéma de signature et un extrait de vérification sont dans la documentation Hook0."
      },
      {
        "q": "Est-ce que ça marche avec Hook0 auto-hébergé ?",
        "a": "Oui. Mets HOOK0_API_URL sur ton instance et les dix-sept outils se comportent à l'identique. Le produit entier est à code source ouvert (SSPL-1.0) sans rétention open-core : le déploiement auto-hébergé fait tourner le même logiciel que le cloud."
      },
      {
        "q": "Est-ce un Agent Skill ou un plugin ?",
        "a": "Non. C'est un serveur MCP, installé avec cargo install hook0-mcp et déclaré dans la configuration de ton assistant. Il tourne en stdio par défaut, ou en SSE si tu préfères le faire tourner en service."
      },
      {
        "q": "Ai-je besoin du serveur MCP pour envoyer des webhooks ?",
        "a": "Non. L'API REST et les SDK restent le chemin normal pour que ton application émette ses events, et rien de tout ceci ne les affecte. Le serveur MCP s'adresse à la personne qui exploite l'installation : déclarer un type d'event, brancher une souscription, comprendre pourquoi une livraison a échoué à seize heures."
      }
    ]
  },
  "related": {
    "h2": "Pour aller plus loin",
    "links": [
      { "href": "/fr/api-webhook", "label": "API webhook" },
      { "href": "/fr/plateforme-webhook", "label": "Plateforme webhook" },
      { "href": "/fr/webhooks-auto-heberges", "label": "Webhooks auto-hébergés" },
      { "href": "/fr/infrastructure-webhook-europeenne", "label": "Infrastructure webhook européenne" },
      { "href": "https://documentation.hook0.com/reference/mcp", "label": "Documentation du serveur MCP" }
    ]
  }
};
