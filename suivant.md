# prochain point: 

SPAWN fait en sorte que toutes les pages au sein de l'app (celle qui sont sous /organizations/*) ait un breadcrumb et que ce
 breadcrumb permet de révenir à la page /. Le breadcrumb doit être détaillé par exemple il donne:

[lien vers /: "Organisations"] / [logo d'une orga] [lien vers l'orga: "FGRibreau SARL"] / [logo d'une app] [lien vers l'app "Image-Charts"] / [lien vers "Subscriptions"] / Subscription Details

et ce type de breadcrumb doit être sur TOUTES les pages

SPAWN mets à jour le CLAUDE.md du frontend pour intégrer ce requirement sur les breadcrumbs

SPAWN les placeholders dans les champs Hook0Input et équivalent sont en noir au lieu d'être en gris clair

---

SPAWN quand un bouton à une icône, ou quand un listitem a une icone, il ne doit pas y avoir de word-wrap/retour à la ligne, ça doit toujours être sur une seule ligne, c'est un hard requirement, corrige

SPAWN des boutons comme le "data-test="delete-account-button"" ont disparu/ne sont plus visibles, identifie pourquoi, idenfie les autres endroit où cela a pu se produire et corrige partout, ajoute un test de non régression visuel sur le component Hook0Button

SPAWN déplace les menu "documentation" et "api reference" dans le menu 

SPAWN le "new subscription" dans l'empty-screen de request attempts doit emmener sur ./subscriptions/new et pas uniquement sur /subscriptions vérifie qu'il n'y a pas aussi ce problème sur les autres CTA des empty screen (je pense que c'est le cas) et corrige

