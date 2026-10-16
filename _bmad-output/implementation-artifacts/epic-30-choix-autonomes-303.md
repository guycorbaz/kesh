# Epic 30 — registre des choix autonomes de la Story 30-3 (à verser par union dans `epic-30-choix-autonomes.md`)

Fiche : `30-3-chemin-facturation.md`. Issue [#585]. Choix tranchés au développement, le 2026-10-16, en autonomie.

[#585]: https://github.com/guycorbaz/kesh/issues/585

---

### C-30-3-1 — Corriger le texte, ne pas ajouter de lien

- **Contexte** : l'issue offre deux remèdes — écrire le vrai chemin, ou ajouter sur `/settings` un lien vers la facturation.
- **Retenu** : écrire le vrai chemin (« Administration → Facturation »). Le lien aurait changé la navigation d'un seul
  écran parmi plusieurs entrées sœurs (Exercices, Taux de TVA, Utilisateurs…) qui ont le même défaut.
- **Écartées** : le lien sur `/settings` (périmètre de navigation, à décider pour l'ensemble des entrées).
- **Réversible** : oui.

### C-30-3-2 — Le titre de l'écran reste « Paramètres — Facturation »

- **Contexte** : `settings-invoicing-title` est le titre affiché par l'écran, avec un tiret cadratin : ce n'est pas un chemin.
- **Retenu** : ne pas le toucher (changement d'écran, hors correction de texte de chemin).
- **Écartées** : le renommer « Facturation » (cohérent avec la barre latérale, mais touche les quatre catalogues, des
  tests et des captures ; à prendre avec le lien éventuel de C-30-3-1).
- **Réversible** : oui.

### C-30-3-3 — La garde G3 suit la barre latérale

- **Contexte** : `le_message_du_compte_de_produit_renvoie_a_l_ecran_reel` (`kesh-i18n/src/loader.rs`) dérivait le chemin
  attendu du titre d'écran (` — ` → ` → `), ce qui entérinait « Paramètres → Facturation ».
- **Retenu** : la garde construit le chemin depuis `nav-administration` et `nav-invoicing` de chaque locale ; une
  clé de navigation renommée fait rougir le message.
- **Écartées** : un chemin écrit en dur par locale (ne suit pas le renommage d'une entrée de menu).
- **Réversible** : oui.
