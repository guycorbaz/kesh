# Epic 34 — choix autonomes du développement de la 34-9 (clé `e349`)

Story `34-9-rapport-import-dossier` (#969, #974 intégrée). Les choix de la fiche sont au registre du groupe,
`e34-choix-autonomes-e342.md` (C-e342-10 à C-e342-14, C-e342-19 à C-e342-22). Ceux-ci sont nés au développement,
le 2026-11-10 (Opus 5.5). Tous signalés à Guy dans le rapport de l'agent.

### C-e349-1 — Une relecture de doublon en échec n'abat pas l'import

- **Contexte** : aux deux sites de course, le document se relit par son empreinte après le `rollback` (C-e342-10).
  Une erreur de base à ce moment-là laisserait le fichier dans le dossier si elle remontait en `500`.
- **Retenu** : l'erreur est journalisée (`tracing::warn!`) et traitée comme « introuvable » : la ligne dit « Déjà
  importé. », sans lien, avec `movedTo` (`reread_duplicate`, `inbox_import.rs`).
- **Écartée** : `?` (un `500` global pour une information de rapport ; le fichier resterait et serait retraité).
- **Réversible** : oui.

### C-e349-2 — Résumé d'un import sans aucun fichier

- **Contexte** : AC-4 omet les segments nuls ; un import vide n'en aurait aucun.
- **Retenu** : le segment « document(s) importé(s) » reste quand tous les autres sont nuls : « 0 document(s)
  importé(s). », comme l'écran d'avant.
- **Écartée** : un texte dédié (« Aucun fichier dans le dossier. ») — une clé de plus, hors fiche.
- **Réversible** : oui.

### C-e349-3 — Libellés d'état, et deux clés de plus que la fiche

- **Contexte** : la fiche nomme les clés du résumé et décrit les textes des lignes ; elle ne nomme pas toutes les
  clés des états.
- **Retenu** : chaque état commence par un libellé capitalisé — « Importé. », « Importé sans les données du QR : »,
  « Déjà importé : », « Refusé : », « Non traité : » — d'où `…-report-state-imported`, `…-state-imported-no-qr`,
  `…-state-not-processed`, `…-report-refused`, `…-report-duplicate` et ses variantes (`-unknown`, `-pending`,
  `-invoice`, `-discarded`, `-processed`). Toutes sont aux quatre catalogues et au test du catalogue réel.
- **Écartée** : réutiliser `importErrorLabel('DUPLICATE')` (« Déjà importé (doublon) ») : la ligne dit déjà où est
  le document.
- **Réversible** : oui (textes).

### C-e349-4 — Les anciens testids portés par l'intérieur de la ligne

- **Contexte** : AC-4 veut `inbox-import-file-row` et, « sur les lignes concernées », les anciens testids ; un élément
  ne porte qu'un `data-testid`.
- **Retenu** : `<li data-testid="inbox-import-file-row" data-state=…>` contient un `<div>` qui porte
  `inbox-import-accepted-row` ou `inbox-import-failed-row` et tout le texte de la ligne : `getByTestId(…).filter({
  hasText })` et `textContent` lisent la même chose qu'avant.
- **Réversible** : oui.

### C-e349-5 — Les clés `…-report-accepted` et `…-report-failed` retirées

- **Contexte** : remplacées par le résumé composé ; la fiche (T3) laisse « gardées ou retirées avec leur garde ».
- **Retenu** : retirées des quatre catalogues ; aucun code ne les lit plus.
- **Réversible** : oui.

### C-e349-6 — Les deux courses et l'instabilité se montent sans crochet de production

- **Contexte** : AC-3 demande les courses si elles se montent sans crochet, et l'instabilité si le montage le permet.
- **Retenu** : (a) course de la réactivation manquée — une transaction du test met la ligne `to_complete` sans
  valider ; la lecture de l'import (non verrouillante) voit `discarded`, son `UPDATE … WHERE status='discarded'`
  attend le verrou, le test valide, l'`UPDATE` ne touche rien ; (b) course `UNIQUE` — une transaction du test insère
  la même empreinte sans valider, l'`INSERT` de l'import attend puis échoue en doublon. L'attente se lit dans
  `information_schema.PROCESSLIST` (ses propres sessions, sans droit `PROCESS`), avant de valider. (c) instabilité — un
  fil ajoute un octet au fichier chaque milliseconde pendant l'import. `VANISHED` ne se monte pas sans crochet : il est
  couvert par le test unitaire de `push_outcome`.
- **Écartée** : un crochet de test dans `inbox_import.rs`.
- **Réversible** : oui (tests).

### C-e349-7 — Rechargement en échec : une notification, deux lignes

- **Contexte** : F9-8, « une seule notification d'avertissement, qui dit les deux ».
- **Retenu** : `notifyWarning(message de l'import, description = « La liste n'a pas pu être rechargée — actualisez la
  page. »)` (clé existante `imported-supplier-invoices-reload-failed`) ; sans message d'import (import vide), le texte
  existant `…-completed-reload-failed`.
- **Réversible** : oui.

### C-e349-8 — Couleurs : bordure d'avertissement, aucun texte ambre

- **Contexte** : `--color-warning` (`#d97706`) sur blanc n'atteint pas le contraste d'un petit texte.
- **Retenu** : l'encadré porte une bordure de 2 px `border-warning` quand il contient un échec ou un fichier non
  traité ; les libellés d'état sont en gras, « Refusé : » en `text-destructive` ; aucun texte en `text-warning`.
- **Réversible** : oui.

### C-e349-9 — Un `details` qui n'est pas un objet est gardé

- **Contexte** : `movedTo` se fusionne dans `details` (F9-2) ; tous les `details` d'aujourd'hui sont des objets.
- **Retenu** : un `details` non objet serait gardé sous `detail`, à côté de `movedTo` (`with_moved_to`), jamais perdu.
- **Réversible** : oui.

### C-e349-10 — Ordre des noms

- **Contexte** : C-e342-12, « tri par nom à l'écran ».
- **Retenu** : `localeCompare(…, 'fr', { numeric: true, sensitivity: 'base' })`, puis l'ordre des points de code pour
  départager — « facture-2 » avant « facture-10 », sans égard à la casse.
- **Réversible** : oui.
