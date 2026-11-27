# Story 49-2 : réordonner les lignes d'une facture client en brouillon — boutons ↑ / ↓, au clavier (#1756)

Status: in-progress

*Fiche rédigée le 2026-11-27 (Opus 5.5) par le workflow `bmad-create-story`, sur décision du Project Lead du 2026-11-27 :
la 49-2 est la fonctionnalité de la **v0.20.1** (milestone `v0.20.1`, échéance lundi 2026-11-30 ; la fiche disait d'abord
v0.21.0, vendredi 2026-12-04 : corrigé à la remédiation de la revue P1, décision du Project Lead sur #1756), seule story de l'**Epic 52** réduit avec sa story de solde (`epic-52-lignes-de-facture-suite.md`). **Aucune
migration.** Code lu à la tête de la branche `story/49-2-reordonnancement-lignes` = `main` **`0e99cd2db`** (la v0.20.0 :
types de lignes `priced`, `comment`, `heading` de l'Epic 49 ; `heading` encore fermé à l'API). Sources :
`epic-49-types-de-lignes-facture.md` (§ 49-2, § « Ce qui se lit au code », tableau des fichiers), #1242 (section
« Réordonnancement »), les fiches closes `49-1a-types-de-lignes-migration-api.md` et `49-1c-types-de-lignes-ecran.md`
(Dev Notes : « le mapper `invoice-lines.ts` est le point d'appui de la 49-2 »). Choix de la fiche : D1 à D12. Clé courte
des bases et journaux : **`e492`**.*

**Issues** (décisions du Project Lead du 2026-11-27) : la story **ferme #1756** (« Facture client : réordonner les lignes
d'un brouillon (↑/↓) », milestone `v0.20.1`), issue propre au réordonnancement, et cite **#1242** en `refs` seulement.
Chaque commit porte `refs #1756 refs #1242`, **sans aucun mot-clé de fermeture**, ni dans un commit ni en prose ; le
`closes #1756` ne s'écrit que dans le titre ou le corps de la **PR qui entre dans `main`** (celle de l'epic ou de la
release : une PR vers `epic/52-…` ne ferme rien, GitHub ne fermant une issue qu'à la fusion dans la branche par défaut).
#1242 (milestone `v0.22.0`) ne se ferme pas avec cette story : les titres de section sont à la 49-3 (Epic 62, v0.22.0),
les sous-totaux à #1330. Le glisser-déposer est **#1757** (« Plus tard ») ; la page de modification ouverte sur une
facture validée est **#1758** (P3, hors de la story).

## Story

En tant que Guy, qui émet ses vraies factures clients dans Kesh, je veux **créer toutes les lignes d'une facture à la
suite, puis les remettre dans l'ordre** où elles doivent s'imprimer — une ligne de commentaire au-dessus de l'article
qu'elle explique, la prestation principale en tête —, **sans retaper aucune ligne**, au clavier comme à la souris, tant que
la facture est un **brouillon** ; et je veux qu'une facture **validée** ne change jamais d'ordre, son PDF et son écriture
faisant foi.

## Ce qui se lit au code (`0e99cd2db`)

### L'ordre est porté par `position`, et la position est le rang dans la requête

- `crates/kesh-db/migrations/20260416000001_invoices.sql:44` : `    position INT NOT NULL,` ; `:59` :
  `    CONSTRAINT uq_invoice_lines_position UNIQUE (invoice_id, position),`.
- `crates/kesh-db/migrations/20260627000001_credit_notes.sql:66` (`position INT NOT NULL`) et `:82`
  (`CONSTRAINT uq_credit_note_lines_position UNIQUE (credit_note_id, position)`) : même structure côté avoir.
- `crates/kesh-db/src/repositories/invoices.rs:557` (dans `insert_lines`, `:541`) : `        .bind(idx as i32)` — la
  position est l'**indice de la ligne dans le vecteur reçu**, de 0 à n − 1. Le client n'envoie **aucune** position :
  `frontend/src/lib/features/invoices/invoices.types.ts:28` (`position: number;`) n'est que dans la **réponse** ;
  `CreateInvoiceLineRequest` (`crates/kesh-api/src/routes/invoices.rs:79`) n'a pas de champ `position`.
- `invoices.rs:592` et `:953` : `"SELECT {LINE_COLUMNS} FROM invoice_lines WHERE invoice_id = ? ORDER BY position ASC"` —
  toute relecture (édition, fiche, PDF, validation, MCP) suit la position.

### La modification d'un brouillon remplace toutes ses lignes

- `invoices.rs:1206` `pub async fn update(` : `SELECT … FOR UPDATE` de la facture, puis garde de statut
  (`:1270` : `"impossible de modifier une facture de statut '{}'"`) et de version (`:1274` :
  `Some(inv) if inv.version != expected_version => {`), court-circuit no-op, puis **`:1392`**
  `sqlx::query("DELETE FROM invoice_lines WHERE invoice_id = ?")` et réinsertion par `insert_lines`, enfin `UPDATE
  invoices … version = version + 1 … AND status = 'draft'`. Les anciennes lignes sont **supprimées avant** l'insertion
  des nouvelles, dans la même transaction : la contrainte `uq_invoice_lines_position` ne voit jamais deux lignes à la même
  position. **Réordonner = renvoyer le `PUT` avec les lignes dans le nouvel ordre** : aucun changement du serveur, aucune
  migration (D1, D10).
- `invoices.rs:1157` `fn is_no_op_change(` compare les lignes **dans l'ordre** (`:1173` :
  `before_lines.iter().zip(changes.lines.iter()).all(|(b, c)| {`) : une permutation de lignes différentes **n'est pas** un
  no-op. Le test `invoices.rs:5982` `async fn update_line_reorder_bumps_version()` (KF-004) le garde déjà — mais il
  n'affirme que le `version + 1`, **ni l'ordre relu, ni les positions** (T3 le complète).
- Le journal d'audit porte la position de chaque ligne : `invoices.rs:113` `"position": l.position,` dans
  `invoice_snapshot_json`, écrit en `before` / `after` de l'action `invoice.updated`. Un réordonnancement est donc tracé
  sans action nouvelle (D12).
- Route : `PUT /api/v1/invoices/{id}` (`crates/kesh-api/src/lib.rs:736`), handler `update_invoice`
  (`routes/invoices.rs:879`), enveloppé dans `kesh_db::retry::retry_on_deadlock("invoices::update", …)`. Une facture non
  brouillon rend `409 ILLEGAL_STATE_TRANSITION` (`crates/kesh-api/src/errors.rs:6537-6543`, bras `DbError::IllegalStateTransition` ; `:2779` est celui d'`AppError::IllegalState`, autre chemin), comme le teste déjà
  `invoices.rs:3866` `test_update_rejects_non_draft` (dépôt) — mais aucun test HTTP ne le joue avec un corps réordonné
  (T5).

### Ce que l'ordre touche, et ce qu'il ne touche pas

- **L'écriture de vente ne dépend pas de l'ordre des lignes.** `generate_invoice_journal_lines` (`invoices.rs:1949`)
  agrège le hors-taxe par compte et la TVA par taux dans des `BTreeMap` (`:1968` :
  `let mut ht_by_account: BTreeMap<i64, Decimal> = BTreeMap::new();` ; `:1971` : `let mut vat_by_rate: BTreeMap<Decimal,
  Decimal> = BTreeMap::new();`), itérés par clé croissante ; les sommes de `Decimal` sont exactes. La correction de TVA
  d'une perte ou d'un escompte (`invoice_settlements_write.rs:664`, `kesh-core/src/accounting/vat.rs:184`
  `write_off_vat_shares` → `vat_breakdown_by_rate`, `BTreeMap` à `:130`) et le rapport TVA agrègent de même. **Le
  brief demandait que « l'ordre des écritures suive l'ordre » : il n'y a pas d'ordre des lignes dans l'écriture** ; ce qui
  doit tenir, c'est qu'un réordonnancement **ne change pas** l'écriture (AC9).
- **Le PDF suit l'ordre** : `invoice_pdf_service.rs:289` lit `invoices::find_by_id_with_lines` (ordre de position) ; le PDF
  n'existe que pour une facture validée (`:293` : `if invoice.status != "validated" {`). Il se fige au premier
  téléchargement ou envoi ; la dévalidation le détache (`invoices.rs`, `unvalidate`, `pdf_storage_path = NULL …`), et la
  revalidation en fige un nouveau. Le contrôle de capacité de la 49-4a compte des lignes imprimées, sans égard à leur
  ordre.
- **Les avoirs n'ont pas de brouillon** : `20260627000001_credit_notes.sql:18` (`-- INSERT pose toujours 'issued' (DC5) ;
  'draft'/'cancelled' réservés compat-forward.`) ; aucune route `PUT /credit-notes/{id}` (`lib.rs:741`, `:1316-1325` :
  `POST`, `GET`, `GET …/pdf`). L'avoir copie les lignes de sa facture avec leur position (`credit_notes.rs:456` :
  `"SELECT {} FROM invoice_lines WHERE invoice_id = ? ORDER BY position"`, puis `.bind(line.position)` à l'insertion,
  `:820`). **Un avoir ne se réordonne pas ; il suit l'ordre de sa facture** (D9).
- Exports : `invoices.rs:3167` `ORDER BY il.invoice_id, il.position` (CSV `invoice_lines.csv`) — suit l'ordre, rien à
  changer.

### L'écran (`frontend/src/lib/components/invoices/InvoiceForm.svelte`, 1239 lignes)

- Un seul formulaire pour la création (`routes/(app)/invoices/new`) et la modification d'un brouillon
  (`routes/(app)/invoices/[id]/edit`) ; l'état des lignes est `let lines = $state<LineState[]>(initLines());` (`:250`),
  `LineState = LigneFormulaire` (`features/invoices/invoice-lines.ts`), dont `_uiKey` est « Identifiant d'interface stable ;
  jamais envoyé au serveur » — commenté dès l'origine `// \`_uiKey\` : identifiant UI stable (cross-reorder/remove), jamais
  envoyé au backend.` (`:118`).
- Le tableau est **déjà indexé par la clé stable** : `:1008` `{#each lines as line, i (line._uiKey)}`. Deux rangées par
  type : la rangée de texte (`:1013`, `data-testid="invoice-line-comment-row"`, `colspan={LINE_TABLE_COLUMNS - 1}`) et la
  rangée chiffrée ; **chacune** se termine par la même cellule d'actions, la poubelle recopiée deux fois (`:1034` et `:1112`,
  `onclick={() => removeLine(i)}`, avec `aria-label="Supprimer la ligne"` et `disabled={lines.length <= 1}`). La rangée chiffrée peut être suivie
  d'une rangée de message du prix ambigu ou illisible, dans le même bloc de l'`#each` (`:1121-1149`).
- Colonnes : `const LINE_TABLE_COLUMNS = 7;` (`:566`), en-tête d'actions `<th class="py-2 w-12"></th>` (`:1004`), pied
  `<td colspan="5" class="py-3 text-right font-semibold">Total</td>` (`:1154`).
- Suppression : `function removeLine(i: number) { lines.splice(i, 1); }` (`:540`). Trois ajouts en fin de tableau
  (`addFreeLine`, `addCommentLine`, `onProductSelect`, `:485-538`).
- Envoi : `onSubmit` (`:676`) bâtit `payloadLines` **dans l'ordre de `lines`** (boucle `for (let i = 0; i < lines.length;
  i++)`), puis `updateInvoice(invoice.id, { ...payload, version: invoice.version })` (`:768`) ; un
  `OPTIMISTIC_LOCK_CONFLICT` ouvre la modale de conflit (`conflictOpen`), dont « Recharger » (`reloadFromServer`, `:799`)
  remplace `lines` par celles du serveur.
- Messages qui nomment une ligne par son rang : `validateClient` (`:604`) rend `Ligne ${i + 1} : …` dans `errorMsg`
  (`:270`), qui **reste affiché** tant qu'un envoi ne l'efface pas ; `invalidLinesMessage` (`:402`) est un `$derived`, qui
  se recalcule seul.
- Aucun motif de réordonnancement ni de glisser-déposer n'existe dans le frontend (`git grep -n -E
  "draggable|dragstart|moveUp|ChevronUp|ArrowUp" -- frontend/src` : seulement la touche `ArrowUp` des listes de
  `ContactPicker.svelte:264` et `AccountAutocomplete.svelte:357`, leurs tests, et l'icône de défilement
  `ui/select/select-scroll-up-button.svelte`) ; `git grep -n -E "lines\.(sort|reverse|toSorted)" -- frontend/src` : vide ; aucune bibliothèque de glisser-déposer dans `frontend/package.json` (`svelte` `^5.54.0`,
  `@lucide/svelte` `^1.7.0`).
- La fiche de la facture (`routes/(app)/invoices/[id]/+page.svelte`) ne montre le bouton « Modifier » qu'à un brouillon
  (`:1068` `{#if invoice?.status === 'draft'}`, `:1074` `goto(\`/invoices/${invoice!.id}/edit\`)`). La page d'édition, elle,
  **ne contrôle pas le statut** (`routes/(app)/invoices/[id]/edit/+page.svelte` : `getInvoice` puis `<InvoiceForm
  {invoice} />`) : ouverte par son URL sur une facture validée, elle montre le formulaire, et l'enregistrement est refusé
  par le serveur (`409`). Défaut existant, hors périmètre : **#1758** (P3).

### Les textes qui deviendront faux

- `docs/manual/fr/user-manual.tex:1572` : « Elle se place par l'ordre des ajouts~; son type ne change plus » (au sujet
  de la ligne de commentaire), et « Trois boutons, sous le tableau, ajoutent une ligne en fin de tableau ». Le PDF
  aplati le porte aussi (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` : « Elle se place par
  l’ordre des ajouts ; son type ne change plus »).
- `docs/api-external.md:597-599` (§ « Les lignes d'une facture client — le type de ligne ») : ne dit pas que la position
  est le rang dans `lines`, ni comment réordonner un brouillon.
- Les seuls manuels existants sont en français (`docs/manual/{de,en,it}/` ne portent qu'un `README.md`).

## Inventaire par le symptôme

Le symptôme : **un ordre de lignes de facture qui ne serait pas celui que l'utilisateur a choisi** (perdu à l'envoi, à la
relecture, à l'impression, dans l'avoir), ou un ordre qui **changerait** ce qui ne doit pas l'être (écriture, totaux, TVA).

S1 — qui lit les lignes, et dans quel ordre :

```
$ git grep -n -E "ORDER BY [a-z_.]*position" -- crates/ | grep -v "/tests/"
crates/kesh-db/src/repositories/credit_notes.rs:106:    " FROM credit_note_lines WHERE credit_note_id = ? ORDER BY position"
crates/kesh-db/src/repositories/credit_notes.rs:456:            "SELECT {} FROM invoice_lines WHERE invoice_id = ? ORDER BY position",
crates/kesh-db/src/repositories/invoices.rs:592:        "SELECT {LINE_COLUMNS} FROM invoice_lines WHERE invoice_id = ? ORDER BY position ASC"
crates/kesh-db/src/repositories/invoices.rs:953:        "SELECT {LINE_COLUMNS} FROM invoice_lines WHERE invoice_id = ? ORDER BY position ASC"
(+ payment_batches.rs, supplier_invoices.rs : autres tables, hors sujet)
$ git grep -n "ORDER BY il" -- 'crates/*/src/*'
crates/kesh-db/src/repositories/invoices.rs:3167:         ORDER BY il.invoice_id, il.position"
```

S2 — qui lit `invoice_lines` **sans** ordre :

```
$ git grep -n -E "invoice_lines|credit_note_lines" -- 'crates/*/src/*' | grep -iE "SELECT" | grep -v "ORDER BY"
crates/kesh-db/src/repositories/invoice_settlements_write.rs:664:            "SELECT line_total, vat_rate FROM invoice_lines \
crates/kesh-db/src/repositories/invoices.rs:575:            "SELECT {LINE_COLUMNS} FROM invoice_lines WHERE id = ?"
crates/kesh-db/src/repositories/invoices.rs:4003:  (test)
crates/kesh-db/src/repositories/letterings/open_items.rs:682, :691 (existence seule)
```

Les sites ensemble clos **qui ne résolvent pas l'ordre** et leur sort :

| Site | Résout-il l'ordre ? | Sort |
|---|---|---|
| `invoice_settlements_write.rs:664` (TVA d'une perte ou d'un escompte) | non (pas d'`ORDER BY`) | **sans effet** : agrégation par taux (`vat_breakdown_by_rate`, `BTreeMap`) ; ne s'applique qu'à une facture validée, qui ne se réordonne plus |
| `invoices.rs:575` (relecture d'une ligne par `id`) | sans objet (une ligne) | — |
| `open_items.rs:682`, `:691` | sans objet (`EXISTS`) | — |
| `generate_invoice_journal_lines` (`invoices.rs:1949`) | ignore l'ordre par construction | AC9 le prouve (T4) |
| lecteurs de `find_by_id_with_lines` (`invoice_email.rs:272`, `:431`, `:778`, `:1263`, `dunning_reminders.rs:405`, `invoice_pdf.rs:90`, `invoice_pdf_service.rs:289`, MCP `services/factures.rs:311`, `:575`) | oui, par le dépôt (`ORDER BY position`, `invoices.rs:953`) | rien (V1-4 : S2 filtrait `SELECT` sur la même ligne ; sans filtre, restent des agrégats `SUM`/`GROUP BY` — `invoices.rs:250`, `:263` —, l'`UPDATE` du compte de produit à la validation — `:2426` — et les exports `list_all_lines_by_company`, ordonnés par `position`) |
| frontend : écrans qui lisent `invoice.lines` ou `creditNote.lines` | prennent l'ordre de la réponse (`git grep -n -E "lines\.(sort|reverse|toSorted)" -- frontend/src` : vide à `0e99cd2db`) | à rejouer à l'étape 0 : tout tri trouvé est un finding à traiter |

Angle mort assumé : un client de l'API qui enverrait les lignes dans un ordre arbitraire obtient cet ordre ; c'est le
contrat (D1), écrit au guide de l'API.

## Décisions

- **D1 — Aucun changement du serveur : l'ordre est celui du tableau `lines` du `PUT`.** Le serveur réécrit déjà toutes les
  lignes d'un brouillon à chaque `PUT`, positions de 0 à n − 1 dans l'ordre reçu, sous le verrou et la version de la
  facture. *Écarté* : une route dédiée (`PATCH /invoices/{id}/lines/order` avec une liste d'identifiants) — un second chemin
  d'écriture des lignes, à garder cohérent avec le premier (statut, version, audit, interblocages, capacité), qui devrait
  contourner `uq_invoice_lines_position` (positions provisoires ou `UPDATE` en deux temps) ; et les identifiants de ligne
  changent à chaque `PUT` (réinsertion), ce qui la rendrait fragile. Le réordonnancement **voyage avec l'enregistrement** du
  brouillon, comme toute autre modification.
- **D2 — Les boutons ↑ / ↓ vivent dans la cellule d'actions existante, écrite une fois.** La cellule de la poubelle,
  aujourd'hui recopiée dans les deux rangées (`:1034`, `:1112`), devient un `{#snippet actionsDeLigne(i)}` unique
  (↑, ↓, poubelle), rendu par la rangée de texte et la rangée chiffrée (DRY ; un titre `heading` de la 49-3 en héritera).
  L'en-tête `:1004` s'élargit (`w-12` → la largeur des trois boutons, à fixer à la recette visuelle). *Écarté* : une
  colonne neuve « Ordre » — elle obligerait à toucher `LINE_TABLE_COLUMNS`, le `colspan` de la rangée de texte, celui des
  messages du prix et le `colspan="5"` codé en dur du pied (`:1154`), pour aucun gain.
- **D3 — Pas de glisser-déposer dans cette story.** Le fichier d'épique le permettait « si la fiche le juge sûr » ; elle ne
  le juge pas sûr à cette release : aucune bibliothèque n'est en place, le glisser-déposer HTML5 ne fonctionne ni au
  clavier ni au toucher (il faudrait de toute façon les boutons), il entre en concurrence avec la sélection de texte dans
  les champs de la ligne (description, commentaire multiligne), et les gestes de glisser sous Playwright sont la
  première source de tests instables. Les boutons couvrent le besoin de #1242 (« créer toutes les lignes à la suite puis
  les réordonner »). *Écarté* : glisser-déposer maison sur `pointerdown` (coût, accessibilité à refaire). Le
  glisser-déposer est suivi par **#1757** (milestone « Plus tard », décision du Project Lead du 2026-11-27).
- **D4 — Une fonction pure dans `invoice-lines.ts`.** `deplacerLigne<T>(lignes: readonly T[], index: number, sens: -1 | 1):
  T[]` rend un **nouveau** tableau où la ligne `index` a échangé sa place avec sa voisine ; un `index` hors bornes ou un
  déplacement au-delà d'une extrémité rend une **copie inchangée** (jamais d'exception, jamais de ligne perdue ni
  dupliquée). Le composant fait `lines = deplacerLigne(lines, i, sens)` : les objets de ligne (des proxys `$state`) sont
  déplacés, non recopiés, et chaque ligne garde son `_uiKey`, donc son élément DOM et l'état interne de son
  `AccountAutocomplete`. *Écarté* : `lines.splice` en place dans le composant (non testable seul, et un échange par
  `splice` mal écrit perd une ligne en silence).
- **D5 — Le focus suit la ligne, et le déplacement s'annonce.** Après un déplacement, le focus revient (après `tick()`) sur
  **le même bouton de la ligne déplacée**, retrouvé par sa clé (`data-line-key={line._uiKey}`, `data-move="up|down"`) ; si
  ce bouton est devenu désactivé (la ligne est arrivée en tête ou en fin), le focus va à **l'autre** bouton de la même
  ligne. Une zone `aria-live="polite"`, toujours présente dans le DOM (`data-testid="invoice-lines-move-status"`), annonce
  « Ligne déplacée en position { $position } sur { $total } » ; elle est **hors du `<table>`** (classe `sr-only`), et son texte est **vidé puis reposé** (`tick()` entre les deux) pour qu'un second déplacement vers la même position soit ré-annoncé (V1-7). Les boutons sont des `<Button type="button">` natifs (Tab,
  Entrée, Espace), avec `aria-label` « Monter la ligne { $n } » / « Descendre la ligne { $n } » et le même texte en
  `title` ; leurs icônes sont `ArrowUp` et `ArrowDown` de `@lucide/svelte` (`aria-hidden="true"`, comme la poubelle
  `Trash2`), pour que l'écran montre les flèches que le manuel écrit `$\uparrow$` / `$\downarrow$` (V2-4). *Écarté* : des raccourcis Alt+↑ / Alt+↓ — non découvrables, et en conflit avec des gestes de lecteurs d'écran.
- **D6 — Quand les boutons sont désactivés.** ↑ de la première ligne, ↓ de la dernière ; les deux quand la facture n'a
  qu'une ligne ; tous pendant un envoi (`submitting`) et tant que la modale de conflit est ouverte (`conflictOpen`) —
  l'envoi a déjà bâti sa charge, et un déplacement pendant l'attente ne partirait pas.
- **D7 — Un déplacement efface `errorMsg`.** Le bandeau peut nommer une ligne par son **ancien** rang (« Ligne 3 :
  description requise ») ; après un déplacement il deviendrait faux. L'enregistrement suivant revalide et nomme la ligne à
  son nouveau rang. `invalidLinesMessage` (dérivé) se recalcule seul. Le bandeau porte aussi des messages qui ne
  nomment aucune ligne (« Choisissez le client dans la liste », `InvoiceForm.test.ts:1600`, refus du serveur) : ils
  s'effacent de même, **et c'est accepté** — l'enregistrement suivant les reproduit tant que leur cause demeure, et rien
  ne part entre-temps (V2-3). *Écarté* : renuméroter le message (il faudrait analyser du texte traduit) ; n'effacer que
  les messages de ligne (il faudrait typer `errorMsg`, pour un gain nul).
- **D8 — Une facture validée ne se réordonne jamais, et c'est le serveur qui le garantit.** L'écran n'offre la modification
  qu'à un brouillon (fiche, `:1068`) ; le `PUT` d'une facture validée — corps réordonné compris — rend `409
  ILLEGAL_STATE_TRANSITION` sans rien écrire (T5). Pour réordonner une facture validée, il faut la **dévalider** quand Kesh
  le permet (ni réglée, ni créditée, ni rappelée, ni envoyée par e-mail…), ce qui détache son PDF figé ; la revalidation en
  fige un nouveau. Ce chemin existe déjà ; la story ne l'élargit pas. **Il ne se recommande pas pour autant** : le manuel
  réserve la dévalidation aux erreurs de saisie, et une facture que le client détient (Kesh ne voit que ce qu'il a envoyé
  lui-même : un PDF ou un papier remis à la main ne bloque rien) se corrige par un avoir. Le texte du manuel (AC12) le dit
  et renvoie à l'encadré « Avoir ou dévalidation ? » (V1-1).
- **D9 — Les avoirs ne sont pas concernés.** Ils sont émis sans brouillon et copient l'ordre de leur facture ; T4 vérifie
  qu'un avoir tiré d'une facture réordonnée suit le nouvel ordre.
- **D10 — Aucune migration.** La contrainte `uq_invoice_lines_position` ne gêne pas : le `DELETE` de toutes les lignes
  précède leur réinsertion dans la même transaction (`invoices.rs:1392`, puis `insert_lines`). Elle ne gênerait qu'un
  échange de positions **en place** (`UPDATE … SET position`), que D1 écarte.
- **D11 — Échanger deux lignes identiques est un no-op, et c'est juste.** `is_no_op_change` compare dans l'ordre : deux
  lignes **strictement identiques** échangées ne changent rien (réponse `200`, même `version`) ; le PDF serait le même.
  L'écran annonce « Facture modifiée » comme pour tout enregistrement sans changement.
- **D12 — La trace d'audit existante suffit.** `invoice.updated` porte `before` et `after` avec la position de chaque ligne
  (`invoices.rs:113`) ; aucune action d'audit neuve. T4 l'affirme.

## Critères d'acceptation

1. **AC1 — Les boutons.** Dans le formulaire de facture (nouvelle facture et brouillon rouvert), chaque ligne — chiffrée
   comme de commentaire — porte un bouton ↑ et un bouton ↓ dans sa cellule d'actions, à côté de la poubelle ; ↑ est
   désactivé sur la première ligne, ↓ sur la dernière, les deux sur une facture d'une seule ligne. *Constat* : T2 ; captures
   C1, C2.
2. **AC2 — Déplacer ne perd rien.** Un clic sur ↑ (↓) échange la ligne avec la précédente (suivante) ; chaque ligne garde
   sa description, sa quantité, son prix (y compris une saisie ambiguë ou illisible et son message, qui la suivent), son
   taux, son compte de produit et son type ; le total ne change pas. *Constat* : T1, T2.
3. **AC3 — L'ordre affiché est l'ordre enregistré.** « Enregistrer » envoie les lignes dans l'ordre affiché ; rouvrir le
   brouillon les montre dans cet ordre ; le serveur leur donne les positions 0 à n − 1 dans cet ordre. *Constat* : T2 (corps
   envoyé), T3 (dépôt), T4 (HTTP), T6 (navigateur).
4. **AC4 — Clavier et lecteur d'écran.** Les boutons s'atteignent par Tab et s'activent par Entrée ou Espace ; leur nom
   accessible est « Monter la ligne N » / « Descendre la ligne N » (N = rang affiché, à jour après chaque déplacement) ;
   après un déplacement, le focus est sur le même bouton de la ligne déplacée, ou sur l'autre bouton de la même ligne si
   celui-ci est désactivé ; la zone `aria-live` annonce « Ligne déplacée en position P sur T ». *Constat* : T2, T6 ; capture
   C3 (anneau de focus visible).
5. **AC5 — Pendant un envoi ou un conflit**, les boutons de déplacement sont désactivés ; un déplacement efface le bandeau
   `invoice-form-error`. *Constat* : T2.
6. **AC6 — Conflit de version.** Si le brouillon a changé ailleurs, l'enregistrement d'un ordre neuf ouvre la modale de
   conflit (`409 OPTIMISTIC_LOCK_CONFLICT`) ; « Recharger » remplace l'ordre local par celui du serveur. *Constat* : T2.
7. **AC7 — Une facture validée ne se réordonne pas.** `PUT /api/v1/invoices/{id}` sur une facture validée, avec ses lignes
   dans un autre ordre, rend `409 ILLEGAL_STATE_TRANSITION` ; ses lignes, leurs positions, sa `version`, son écriture et son
   PDF figé restent identiques. *Constat* : T5.
8. **AC8 — Le PDF et l'avoir suivent l'ordre.** Un brouillon réordonné puis validé s'imprime dans le nouvel ordre (de haut en
   bas) ; l'avoir qui l'annule reprend cet ordre, à l'écran de l'API comme sur son PDF. *Constat* : T4.
9. **AC9 — L'écriture ne dépend pas de l'ordre.** Deux factures aux mêmes lignes dans deux ordres différents produisent, à
   la validation, la même écriture de vente (mêmes comptes, mêmes montants, mêmes sens, même nombre de lignes), le même
   total, la même TVA. *Constat* : T4.
10. **AC10 — Trace d'audit.** Le réordonnancement d'un brouillon écrit une action `invoice.updated` dont `before` et `after`
    portent l'ancien et le nouvel ordre (positions). *Constat* : T4.
11. **AC11 — Aucune facture ni aucun avoir existants ne change.** Le diff de la story ne touche **aucun code de production
    sous `crates/`** : `git diff --stat origin/epic/52-lignes-de-facture-suite...HEAD -- 'crates/*/src/*'` ne nomme que
    `kesh-db/src/repositories/invoices.rs` (un fichier de production **qui porte son module de tests** : le nom du fichier ne
    prouve rien), et **chaque** hunk de celui-ci commence après la ligne `mod tests {` du fichier (constat : `git diff -U0
    origin/epic/52-lignes-de-facture-suite...HEAD -- crates/kesh-db/src/repositories/invoices.rs | grep '^@@'` — tous les
    numéros de ligne d'ancre supérieurs à celui de `mod tests {`, relevé par `grep -n '^mod tests' …` ; `3180` à
    `0e99cd2db`) ; les catalogues se contrôlent à part (`git diff --stat … -- crates/kesh-i18n/locales/`, hors du chemin
    `src` : le premier motif ne les voit pas, V1-2) ; ouvrir un brouillon et
    l'enregistrer sans geste envoie le même corps qu'en v0.20.0 (test existant `InvoiceForm.test.ts:1607` « AC9 a », vert
    **sans modification**) ; les tests de rendu et de PDF existants restent verts sans modification (T8). *Constat* : T8 et
    la commande ci-dessus, sortie copiée au Dev Agent Record.
12. **AC12 — Les documents disent vrai.** `user-manual.tex:1572` ne dit plus que la ligne « se place par l'ordre des
    ajouts » : il dit comment réordonner (↑ / ↓, en brouillon, au clavier), que l'ordre est celui du PDF, qu'il se garde à
    l'enregistrement, qu'il ne change ni le total, ni la TVA, ni l'écriture, et qu'une facture validée ne se réordonne
    qu'après dévalidation. `docs/api-external.md` (§ des lignes d'une facture client) dit : la position d'une ligne est son
    rang dans `lines` (à partir de 0) ; réordonner un brouillon, c'est renvoyer le `PUT` avec les lignes dans le nouvel
    ordre (`version` exigée, `version + 1`) ; `409 ILLEGAL_STATE_TRANSITION` sur une facture validée ; l'écriture de vente
    ne dépend pas de l'ordre ; l'avoir le reprend. **Le texte du manuel ne présente pas la dévalidation comme un moyen
    ordinaire de réordonner** : il renvoie à l'encadré « Avoir ou dévalidation ? » (`user-manual.tex:2166-2172`, environnement `keshtip`, § *Supprimer
    ou dévalider une facture*, `:2091` — sous-section **sans `\label`** : en poser un, `\label{sec:devalider-facture}`,
    pour écrire `\S\ref{sec:devalider-facture}`, V2-1) — une facture **envoyée au client** ou **déjà remise** (papier compris), réglée, créditée ou
    d'un exercice déclaré ne se dévalide pas pour une question d'ordre, on émet un avoir (V1-1). *Constat* : PDF du manuel
    régénéré **pour relecture** (non commité), aplati, et grep des phrases neuves et du renvoi (sortie au Dev Agent Record).
13. **AC13 — Textes en quatre langues.** Les trois textes neufs (« Monter la ligne { $n } », « Descendre la ligne { $n } »,
    « Ligne déplacée en position { $position } sur { $total } ») sont des clés `i18nMsg` dans les quatre catalogues
    (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`), dans un bloc « Epic 52 — Story 49-2 » ;
    `sitesTotal` recompté ; `npm run lint-i18n-ownership` vert. *Constat* : T7.
14. **AC14 — Le parcours, dans le navigateur, et la recette visuelle.** E2E « créer toutes les lignes, puis réordonner »
    (T6) ; captures en thème clair, sur une base de démonstration (`kesh_e2e_e492`, jamais celle du NAS), lues une à une et
    jointes au Dev Agent Record et au prompt de la revue de code : **C1** formulaire de trois lignes (bornes désactivées
    visibles), **C2** après déplacement du commentaire en tête, **C3** anneau de focus sur le bouton après un déplacement au
    clavier, **C4** brouillon rouvert dans le nouvel ordre, **C5** fiche de la facture validée dans le nouvel ordre (sans
    boutons), **C6** PDF de cette facture rendu en PNG (`mutool draw -r 150`).

## Tests à écrire, et la preuve du rouge

Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e492/` (`rouge-T<n>.log` : la mutation, la commande, la sortie rouge).
Chaque mutation s'ajoute, le test tourne, le journal se garde, puis la mutation se retire **et le fichier se touche**
(`touch`) avant la reconstruction (cargo garderait le binaire muté) ; côté frontend, `git diff` vide sur le fichier muté,
copié au journal. Une mutation de code de production (T3, T4, T5) n'existe **que le temps du journal**.

| # | Test | Ce qu'il prouve | Preuve du rouge |
|---|---|---|---|
| T1 | `frontend/src/lib/features/invoices/invoice-lines.test.ts` (cas neufs `deplacerLigne`) | AC2, D4 : ↑ et ↓ au milieu, aux deux bornes (copie inchangée), index hors bornes, tableau d'une ligne ; le tableau rendu est neuf (`not.toBe`) et contient **les mêmes objets** (`toBe` élément par élément) ; le tableau d'entrée n'est pas modifié | mutation : l'échange recopie la ligne au lieu de la déplacer (`[a, a]`) → **ROUGE** ; mutation : la borne haute accepte `index + 1 === length` → **ROUGE** ; mutation : `splice` sur le tableau d'entrée → **ROUGE** (`rouge-T1.log`) |
| T2 | `frontend/src/lib/components/invoices/InvoiceForm.test.ts` (nouveau `describe` « Story 49-2 ») | AC1 : états désactivés aux bornes et à une ligne ; AC2-AC3 : brouillon `[A chiffrée, compte X ; C commentaire ; B chiffrée, compte Y]`, ↓ sur A deux fois → corps de `updateInvoice` = `[C, B, A]` avec les comptes et types de chacun, total inchangé ; AC4 : `aria-label` renuméroté, `document.activeElement` après le déplacement (même bouton, puis l'autre à la borne), texte de la zone `aria-live` ; AC5 : boutons désactivés pendant l'envoi (promesse de `updateInvoice` tenue en suspens) et après `OPTIMISTIC_LOCK_CONFLICT` ; `errorMsg` effacé ; AC6 : après « Recharger », l'ordre est celui du serveur | mutation : `moveLine` appelle `deplacerLigne` avec `sens` inversé → **ROUGE** ; mutation : le rappel du focus retiré → **ROUGE** — **sur le cas de la borne** (↑ activé sur la 2e ligne, la ligne arrive en tête, ↑ devient désactivé : sans rappel, `document.activeElement` reste le bouton désactivé) ; le cas « même bouton » ne discrimine pas forcément sous jsdom, qui peut garder le focus d'un nœud déplacé (V1-6) ; mutation : `disabled` de ↑ réduit à `lines.length <= 1` → **ROUGE** ; mutation : `errorMsg = ''` retiré → **ROUGE** (`rouge-T2.log`) |
| T3 | `crates/kesh-db/src/repositories/invoices.rs::tests::update_line_reorder_bumps_version` (complété, non dupliqué) | AC3 au dépôt : la réponse de `update` **et** `find_by_id_with_lines` rendent `[B, A]` (descriptions) aux positions `[0, 1]` | mutation : `ORDER BY position ASC` → `DESC` à `invoices.rs:953` → **ROUGE** ; mutation : `is_no_op_change` compare les lignes triées par description (ensemble, non suite) → **ROUGE** (`rouge-T3.log`) |
| T4 | `crates/kesh-api/tests/invoice_line_types_e2e.rs::reordering_a_draft_moves_the_pdf_not_the_entry` (neuf ; réutilise `monter`, `banque` (compte à QR-IBAN, exigé pour rendre un PDF, `:518-519`, V2-2), `chiffree`, `commentaire`, `positions`, `pdf_200`, `ecriture` ; **deux aides à ajouter** (V1-3) : `monter` n'expose que `caisse` et la fixture ne porte qu'un compte de produit (`3000`) — créer un second compte `Revenue` par SQL, imputable, de la société, et passer un `revenueAccountId` facultatif à une variante de `chiffree` ; sans cela « compte X / compte Y » est inatteignable et l'écriture ne différerait que par ses lignes de TVA) | AC3, AC8, AC9, AC10 par le HTTP : facture 1 `[A (8.1 %, compte X), C commentaire, B (2.6 %, compte Y)]` ; `PUT` en `[B, C, A]` → `200`, `version + 1`, `positions` = `[0, 1, 2]` dans cet ordre, `GET` idem ; audit `invoice.updated` (`before` : A, C, B ; `after` : B, C, A) ; facture 2 créée d'emblée en `[A, C, B]` ; les deux validées : `ecriture()` égales, totaux et TVA égaux ; PDF de la facture 1 : `kesh_qrbill::empreinte::textes` place B au-dessus de C au-dessus de A (ordonnées décroissantes) ; avoir de la facture 1 (`POST /api/v1/credit-notes`) : lignes `[B, C, A]` et PDF de l'avoir dans cet ordre | mutation : `insert_lines` lie `(lines.len() - 1 - idx) as i32` → **ROUGE** sur les positions et le PDF ; mutation : `generate_invoice_journal_lines` agrège dans un `Vec` dans l'ordre d'arrivée au lieu du `BTreeMap` → **ROUGE** sur l'égalité des écritures (`ecriture()`, `invoice_line_types_e2e.rs:815`, lit `journal_entry_lines` `ORDER BY line_order` : l'ordre des lignes d'écriture compte) (`rouge-T4.log`) |
| T5 | `invoice_line_types_e2e.rs::reordering_a_validated_invoice_is_refused_without_writing` (neuf ; `monter`, `banque`, `pdf_200`) | AC7 : facture validée, PDF figé par un `GET …/pdf` ; `PUT` réordonné → `409 ILLEGAL_STATE_TRANSITION` ; `GET` : mêmes lignes, mêmes positions, même `version` ; empreinte du PDF servi inchangée ; aucune action `invoice.updated` neuve | mutation : la garde `Some(inv) if inv.status != "draft"` de `update` (`invoices.rs:1267`) rendue inopérante, **et** la condition `AND status = 'draft'` de l'`UPDATE` retirée → **ROUGE** (`rouge-T5.log`) ; la garde seule retirée rougit déjà sur le code : l'`UPDATE … AND status = 'draft'` ne touche aucune ligne, `update` rend `OptimisticLockConflict` après `rollback` (`invoices.rs:1427-1440`), soit `409 OPTIMISTIC_LOCK_CONFLICT` et rien d'écrit |
| T6 | `frontend/tests/e2e/invoice-line-reorder.spec.ts` (neuf ; mêmes aides que `invoice-comment-lines.spec.ts`) | AC1 à AC4, AC8, AC14 dans le navigateur : nouvelle facture ; « Ligne libre » A ; « Commentaire » C ; « Depuis catalogue » B ; **au clavier** (`focus()` puis `Enter`) : ↑ de C, puis ↓ de A ; ordre attendu `[C, B, A]` ; `toBeFocused()` sur le bouton attendu ; texte de la zone d'annonce ; enregistrer ; rouvrir (édition) : même ordre ; valider ; la fiche montre l'ordre ; `GET …/pdf` `200` ; captures C1 à C6 si `KESH_E2E_CAPTURES_DIR` est posé | mutation : le snippet d'actions rendu sans les boutons de déplacement pour la rangée de commentaire → **ROUGE** ; mutation : `onSubmit` envoie `[...lines].reverse()` → **ROUGE** à la réouverture (`rouge-T6.log`, un seul spec lancé : `npx playwright test invoice-line-reorder`) |
| T7 | `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal` recompté) | AC13 | la borne rougit **d'elle-même** avant le recompte (journal de ce rouge copié) |
| T8 | épreuve, non test neuf : `InvoiceForm.test.ts` « AC9 a » (`:1607`), `invoice-lines.test.ts` existants, `invoice-comment-lines.spec.ts`, `invoices.spec.ts`, `kesh-api/tests/invoice_line_types_e2e.rs` existants, `invoice_pdf_e2e`, `pdf_avant_apres_e2e`, `rendu_avant_apres` | AC11 : verts **sans modification** de leurs assertions | — |

## Tâches

- [x] **Étape 0 — prérequis et inventaire.** Branche à jour de `origin/epic/52-lignes-de-facture-suite` ; rejouer S1, S2 et
  `git grep -n -E "lines\.(sort|reverse|toSorted)" -- frontend/src` ; un écart à la fiche se consigne au Dev Agent Record et
  se traite. Créer les bases `kesh_e492`, `kesh_ci_e492`, `kesh_e2e_e492` sur sa voie (`docs/agents-orchestration.md`,
  `docs/testing.md` § *Base de dev jetable*) — jamais de redémarrage du conteneur depuis un worktree.
- [x] **`invoice-lines.ts`** : `deplacerLigne` (D4), documentée (JSDoc) ; T1.
- [x] **`InvoiceForm.svelte`** : snippet `actionsDeLigne` (D2) rendu par les deux rangées ; `moveLine(i, sens)` (D4, D5, D7)
  avec `tick()` et le rappel du focus ; zone `aria-live` ; états désactivés (D6) ; en-tête `:1004` élargi ; T2.
- [x] **i18n** : trois clés ×4 locales (AC13), bloc « Epic 52 — Story 49-2 » ; `sitesTotal` ; T7 ;
  `npm run lint-i18n-ownership`.
- [x] **`kesh-db`** (module de tests seul) : T3. **`kesh-api`** (tests seuls) : T4, T5 dans `invoice_line_types_e2e.rs`
  (en-tête du fichier mis à jour). **Aucune ligne de production sous `crates/*/src/`** hors des catalogues (AC11).
- [x] **E2E** : T6 ; captures C1 à C6 sur `kesh_e2e_e492` peuplée par le seed de démonstration ; PDF rendu en PNG ; chaque
  capture **lue** ; captures dans `/home/gcorbaz/devel/kesh-gate-logs/e492/captures/` (hors dépôt).
- [x] **Documents** (AC12) : `user-manual.tex:1572` (flèches en `$\uparrow$` / `$\downarrow$`, comme `:1304`) ;
  `docs/api-external.md` (§ « Les lignes d'une facture client ») ; régénérer le PDF du manuel **pour le relire** (`make fr`
  dans `docs/manual/`, sous `scripts/mem-guard.sh`), **sans le commiter** ; aplatir et greper les phrases neuves et
  l'absence de « l'ordre des ajouts ».
- [x] **Propagation** (CLAUDE.md § *Propagation post-patch*) : `git grep -n -E "ordre des ajouts|fin de tableau" -- docs
  frontend crates/kesh-i18n` après le patch ; chaque site restant est juste ou corrigé.
- [ ] **Gates** (CLAUDE.md § *Test Locally First*). **Exception `kesh-db`** : la story touche un fichier de dépôt
  (`repositories/invoices.rs`, module de tests) : **gate complet même en cours de boucle** dès qu'un patch le touche.
  Sinon, en cours de boucle : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo nextest run -E 'binary(kesh_db) | binary(invoice_line_types_e2e)'`, et `npm run check`,
  `npm run lint-i18n-ownership`, `npm run test:unit`. **Au passage en `review`** : bases remises à neuf,
  `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh kesh_ci_e492`, frontend complet (`check`, `lint-i18n-ownership`,
  `test:unit`, `build`). **À la clôture** (dernier commit de code) : `scripts/gate-ci.sh kesh_ci_e492` et **E2E Playwright
  complet** sur `kesh_e2e_e492` recréée (`docs/testing.md` § *Prérequis Playwright local* : `KESH_COOKIE_SECURE=false`,
  `KESH_TEST_MODE=true` des deux côtés, `LANG=fr_CH.UTF-8` installée) ; « vert » = les seuls échecs attendus, jugés
  fichier par fichier (`docs/testing.md` § *Les échecs attendus*). Tout sous `scripts/mem-guard.sh` et
  `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock` ; une commande lourde à la fois.
- [x] **Étapes de coordination** (V1-8, V1-9) : la ligne de la v0.20.1 de la « Feuille de route » de `README.md` (E52, réordonnancement, [#1756](https://github.com/guycorbaz/kesh/issues/1756)) se pose au commit qui la change ; l'issue du réordonnancement est **#1756** (milestone `v0.20.1`, décision du Project Lead du 2026-11-27) : la story la ferme par la PR qui entre dans `main` (`closes #1756` dans son titre ou son corps, jamais dans un commit), et ne fait que citer #1242 (`refs`), qui reste ouverte jusqu'à la 49-3. Le **compte rendu de la PR** nomme #1757 (glisser-déposer) et #1758 (page de modification d'une facture validée), qui restent ouvertes.
- [x] **Contrôle des mots-clés** : `git log origin/epic/52-lignes-de-facture-suite..HEAD --format=%B | grep -niE
  '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` **vide**.

## Note de version

*(Texte pour le `CHANGELOG`, assemblé à l'intégration ; la story n'écrit pas `CHANGELOG.md`.)*

- **Factures — réordonner les lignes d'un brouillon** : chaque ligne du formulaire de facture porte deux boutons ↑ et ↓ qui
  la déplacent d'un rang ; on peut saisir toutes les lignes à la suite — articles, lignes libres, commentaires — puis les
  remettre dans l'ordre où elles s'imprimeront. Les boutons s'utilisent au clavier (Tab, puis Entrée), et le déplacement est
  annoncé aux lecteurs d'écran. L'ordre se garde à l'enregistrement du brouillon et devient celui du PDF et de l'avoir ; il
  ne change ni le total, ni la TVA, ni l'écriture comptable. Une facture validée ne se réordonne pas. Les factures et avoirs
  existants ne changent pas. Aucune migration (#1756).
- **API** : la position d'une ligne est son rang dans le tableau `lines` de `POST` et `PUT /invoices/{id}` ; réordonner un
  brouillon, c'est renvoyer le `PUT` avec ses lignes dans le nouvel ordre (contrat inchangé, désormais écrit au guide)
  (#1756).

## Dev Notes

- **Hors périmètre, avec ce qui le prend** : glisser-déposer → **#1757**, « Plus tard » (D3) ; titres de section
  et ouverture de `heading` → **49-3** (Epic 62, v0.22.0) ; désignation longue et caractères du PDF → **49-4b** (Epic 62) ;
  sous-totaux → **#1330** ; page d'édition d'une facture validée ouverte par son URL → **#1758** (P3) ; réordonner
  depuis la fiche d'un brouillon sans entrer dans l'édition → non demandé.
- **Splitting** (CLAUDE.md § *Règle de splitting préventif*) : modules dont le **code de production** change — le domaine
  `invoices` du frontend (`lib/components/invoices/InvoiceForm.svelte` et `lib/features/invoices/invoice-lines.ts`, un seul
  domaine) et `kesh-i18n` (catalogues) : **deux modules**. `kesh-db` et `kesh-api` ne changent que par leurs tests, qui ne
  comptent pas. Loin du plafond de cinq : pas de découpage, pas de section « Dérogation ».
- **Migration : aucune** (D10). À dire en tête des notes de la v0.20.1 si elle n'en porte pas d'autre.
- **Story de solde de l'Epic 52** (`52-solde-dette-epic`) : plafond de **deux issues** (décision du Project Lead du
  2026-11-27) ; elle ne prend que la dette de la 49-2 (LOW de revue renvoyés en issue).
- **Fichiers communs** : `InvoiceForm.svelte` et `docs/manual/fr/user-manual.tex` avec la **49-3** (Epic 62, v0.22.0), qui
  reprendra `main` après la v0.20.1 et vérifiera qu'un titre se déplace comme un commentaire (le snippet D2 l'y aide) ;
  catalogues i18n ×4 et `sitesTotal` avec tous les epics de la release (blocs par epic, compteur recompté après fusion) ;
  `docs/api-external.md` avec la 49-3 (ouverture de `heading`) et la 49-4b.
- **Ce qui doit rester vrai** : (1) aucune facture ni aucun avoir existants ne change (AC11) ; (2) aucun code de production
  du serveur ne change ; (3) un brouillon ouvert et enregistré sans geste envoie le même corps qu'avant ; (4) une ligne ne
  se perd ni ne se duplique jamais par un déplacement ; (5) le type d'une ligne ne change jamais par un déplacement.
- **Pièges** : (a) **réassigner** `lines` (D4) et non muter un tableau dérivé ; vérifier dans T2 que l'`AccountAutocomplete`
  d'une ligne déplacée montre toujours **son** compte (la clé `_uiKey` le garantit ; une clé par index le casserait) ;
  (b) le focus : un élément déplacé dans le DOM peut perdre le focus selon le navigateur — d'où le rappel explicite après
  `tick()`, à jouer dans T6 (navigateur réel), pas seulement dans jsdom ; (c) la zone `aria-live` doit exister **avant** le
  premier déplacement (une zone créée en même temps que son texte n'est pas annoncée) ; (d) la rangée de message du prix
  ambigu suit sa ligne parce qu'elle est dans le même bloc de l'`#each` : ne pas la sortir du bloc ; (e) T4 doit comparer
  des écritures **réellement** produites par deux ordres différents (deux factures), non une facture à elle-même ;
  (f) la base des captures n'est jamais celle du NAS.
- **Références** : `epic-52-lignes-de-facture-suite.md` ; `epic-49-types-de-lignes-facture.md` (§ « Ce qui se lit au
  code », § 49-2, tableau des fichiers) ; fiche `49-1c-types-de-lignes-ecran.md` (D5 du mapper, Dev Notes, pièges) ;
  `frontend/tests/e2e/invoice-comment-lines.spec.ts` (modèle de l'E2E et des captures) ; `docs/testing.md` ;
  `docs/agents-orchestration.md`.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`bmad-dev-story`), 2026-11-27. Tâches menées dans l'ordre de la fiche ; le passage `ready-for-dev` → `in-progress` n'a pas
été écrit à part au `sprint-status.yaml` (une seule écriture, `review`, à la fin).

### Debug Log References

Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e492/` : `vert-T1.log`, `rouge-T1.log`, `rouge-T2.log`, `rouge-T3.log`,
`rouge-T4.log`, `rouge-T5.log`, `rouge-T6.log`, `rouge-T7.log`, `vert-T3-T4-T5.log`, `t45-premier.log`, `vert-T6.log`, `ac11.log`,
`manuel-grep.log`, `make-fr.log`, `front-build.log` ; captures dans `captures/` ; gates idefix dans `idefix/<id>/` (voir plus bas).

### Completion Notes List

- **Étape 0** : `origin/epic/52-lignes-de-facture-suite` = `0e99cd2db`, la branche part de là. Le détecteur du symptôme
  `git grep -n -E "lines\.(sort|reverse|toSorted)" -- frontend/src` rend **vide** (sonde positive : la même commande sur `\.sort\(`
  rend des sites ailleurs) ; S1 et S2 de la fiche ne sont pas rejoués ligne à ligne (aucun site de lecture n'a changé : le diff de
  production ne touche aucun fichier de lecture, cf. AC11). Base `kesh_e492` créée par `CREATE DATABASE IF NOT EXISTS`, migrée,
  semée ; `kesh_e2e_e492` créée de même ; ni `GRANT`, ni redémarrage du conteneur.
- **`deplacerLigne`** (`invoice-lines.ts`, D4) : fonction pure, copie du tableau, échange des deux voisines, copie inchangée hors
  bornes, aux extrémités et pour un index non entier. T1 : 18 tests verts ; trois mutations rouges (`rouge-T1.log`).
- **`InvoiceForm.svelte`** (D2, D5, D6, D7) : snippet `actionsDeLigne(line, i)` (↑, ↓, poubelle) rendu par les deux rangées ; `moveLine`
  réassigne `lines`, vide `errorMsg`, vide puis repose la zone `aria-live` (`tick()` entre les deux) et rappelle le focus (même
  bouton, ou l'autre s'il est désactivé) ; boutons désactivés aux bornes, à une ligne, pendant l'envoi et pendant le conflit ;
  en-tête d'actions `w-12` → `w-28`. T2 : 10 tests neufs, 86 verts au fichier ; quatre mutations rouges (`rouge-T2.log`).
- **i18n** (AC13) : trois clés × quatre locales, bloc « Epic 52 — Story 49-2 » ; `sitesTotal` 3958 → **3963** (la borne a rougi d'elle-même,
  `rouge-T7.log`) ; `lint-i18n-ownership` vert. Les libellés DE, IT et EN sont ma rédaction, non relue par un natif.
- **Rust, tests seuls** : T3 (`update_line_reorder_bumps_version` complété : réponse de `update` et relecture, `[B, A]` aux positions
  `[0, 1]`), T4 et T5 dans `invoice_line_types_e2e.rs` (aides neuves : `compte_produit_en_plus`, `compte_par_numero`, `chiffree_sur`,
  `descriptions`, `descriptions_audit`, `ordonnee`, `de_haut_en_bas`, `audit_updated`). Mutations rouges : T3 (`ORDER BY position DESC`
  au `find_with_lines_in_tx` ; `is_no_op_change` toujours vrai), T4 (positions inversées à l'insertion ; compte effectif pris à la
  première ligne, soit une écriture qui dépend de l'ordre), T5 (garde de statut inopérante **et** `AND status = 'draft'` retiré de
  l'`UPDATE` de `update`). Écart avec la fiche : la mutation « agrégation dans un `Vec` » de T4 n'a pas été écrite telle quelle ; son
  équivalent (écriture dépendante de l'ordre des lignes) l'a été. Pour T3, la mutation « comparaison par ensemble » a été remplacée
  par « `is_no_op_change` toujours vrai », plus simple à écrire par motif, et qui prouve la même chose (un réordonnancement jugé
  sans effet). **Aucune ligne de production hors module de tests** (AC11, `ac11.log`).
- **E2E T6** (`invoice-line-reorder.spec.ts`) : vert seul (`vert-T6.log`, 1 test) ; mutation A (rangée de commentaire sans boutons)
  rouge à `toBeEnabled`, mutation B (`[...payloadLines].reverse()`) rouge à la réouverture (`rouge-T6.log`). **L'E2E complet n'a pas
  tourné** : il est dû à la clôture, non au passage en `review` (consignes communes de l'Epic 35). Les specs `invoice-comment-lines`,
  `invoices` et le reste de la suite n'ont donc pas été rejoués ici.
- **Documents** (AC12) : `user-manual.tex` (réordonnancement, ordre du PDF et de l'avoir, aucun effet sur total, TVA, écriture ; facture
  validée : dévalidation non recommandée, renvoi à l'encadré *Avoir ou dévalidation ?*) ; `docs/api-external.md` (§ des lignes d'une
  facture client) ; `README.md` (ligne de la v0.20.1). **Écart à la fiche** : la sous-section de l'encadré **porte déjà un label**,
  `sec:suppression-facture` (`user-manual.tex:2092`, la fiche V2-1 le disait sans label) ; le renvoi l'utilise (« §11.12 » au PDF), aucun
  second label n'a été posé. PDF régénéré **pour relecture** (`make fr`, `make-fr.log`), aplati, phrases neuves et renvoi relevés
  (`manuel-grep.log`), puis les trois PDF **restaurés** (`git checkout`) : rien de commité. Propagation : `git grep -n -E "ordre des ajouts|fin de
  tableau" -- docs frontend crates/kesh-i18n` ne rend plus que des sites justes.
- **Recette visuelle** (AC14) : captures C1 à C6 lues une à une, dans `kesh-gate-logs/e492/captures/` (C6 : PDF rendu par
  `mutool draw -r 150`). Prises sur la base `kesh_e2e_e492`, peuplée par le seed `with-company` (jamais celle du NAS), thème clair.
  C1 : bornes grisées (↑ de la 1re ligne, ↓ de la 3e). C3 : anneau de focus sur « Monter la ligne 3 » après le déplacement au clavier.
  C4 : brouillon rouvert dans l'ordre `[commentaire, article B, prestation A]`. C5 : fiche validée, sans boutons. C6 : PDF dans le
  même ordre. Remarque sur C2 : le ↑ de la 2e ligne y paraît grisé, très probablement un fondu de couleur du bouton au moment de la
  capture (le test T2 vérifie l'état `disabled`, et C3 le montre actif) ; non traité.
- **Gates du passage en `review`** : voir la section « Gates » ci-dessous.
- **Remédiation de la revue P1** (Opus 5.5, 2026-11-27, `f4faf30ec` → `a5205bf0f`, `46b3c2ed9`, `2f795d0a7` et suivants ; journaux `$GATE_LOGS/e492/remediation/`) :
  - **Reprise de `main`** (L6, `a5205bf0f`) : un seul conflit de texte, `sitesTotal` ; 3958 + 5 + 7 = **3970**, la garde le confirme (`garde-sites-1.log`).
    La reprise a fait rougir un test venu de `main`, `e2e-selecteurs-traduits.test.ts` (KF-043) : l'E2E visait « Monter la ligne N » par nom
    traduit ; corrigé par des sélecteurs `data-move` (aides `monter(page, n)` et `descendre(page, n)` de la spec).
  - **M1** : paragraphe « Réordonner les lignes » replacé **après** la description du commentaire (« Le commentaire s'imprime… ») ; la contradiction
    de la facture validée est levée (« n'offre pas ces boutons, et le serveur refuse… », renvoi à l'avoir). Vérifié au PDF régénéré dans une copie
    (`make user`, `manuel-grep-2.log`) : les trois phrases périmées comptent 0 occurrence. **M2** : v0.20.1 (milestone de #1756) au guide, au README
    et à la fiche. Les fiches d'epic et `sprint-status.yaml` nomment encore v0.21.0 pour l'Epic 52 : **laissés à l'orchestrateur** (numérotation des
    releases), de même que les prompts des passes passées (historique).
  - **L1** : captures C1 à C6 reprises le 2026-11-27 à 20:45 avec `animations: 'disabled'` (E2E, base `kesh_e2e_e492`, backend lancé par `( cd … && exec env … ) &`,
    port 3492 contrôlé libre, arrêté par son PID) ; C2 montre désormais le ↑ de la 2e ligne foncé. **L2** : `manuel-grep-2.log`. **L3** : mutations de la
    remédiation (`rouge-L7-L8-L10-poubelle.log`, fichier restauré, `cmp` et SHA identiques) ; absence de mutation résiduelle des T1 à T5 :
    `retrait-mutations.log` (seuls hunks de production : aucun, ceux de `invoices.rs` sont aux lignes 6010 et 6029, après `mod tests` en 3180).
  - **L4** : DE « … nach oben verschieben » / « … nach unten verschieben ». **L5** : « à chaque `PUT` qui modifie la facture ». **L7** : test qui observe la zone
    `aria-live` (`MutationObserver`, valeurs successives `[texte, '']`), rouge sans le vidage. **L8** : `ignorerRepetition` (`keydown` répété annulé) sur ↑ et ↓,
    testé, rouge sans lui. **L9** : E2E, touche *Espace*, PDF rendu (`%PDF-`) — l'ordre dans le PDF reste prouvé par T4 (la lecture par `pdftotext`, d'abord écrite, est retirée après la revue P2 : idefix n'a pas l'outil), `expect.poll` pour l'ordre. **L10** : `reloading`
    au verrou (↑, ↓, poubelle, et garde de `moveLine`/`removeLine`), test « Annuler » pendant un rechargement, rouge sans lui. **L11** : README, v0.20.0 « ✅ Publiée ».
  - **Constats antérieurs** : poubelle verrouillée pendant l'envoi, la modale et le rechargement ; libellé `invoice-line-remove` (« Supprimer la ligne { $n } ») dans les quatre
    catalogues, `sitesTotal` **3971** (3970 + 1, l'`aria-label` de la poubelle) ; l'effacement des messages sans rapport avec les lignes reste la **décision D7**, inchangée.
  - **Gates** : `cargo fmt --check` vert ; frontend `check`, `lint-i18n-ownership`, vitest complet **5786 tests**, `build` verts (`front-2.log`) ; build de `kesh-api` et nextest
    ciblé (`kesh-i18n`, `textes_coherents`, `invoice_line_types_e2e`) : **87 tests verts** (`build-api.log`, `nextest-cible.log`) ; E2E `invoice-line-reorder.spec.ts` seule : 1 test vert
    (`e2e-spec-1.log`). Gates de la station sur `190f70eaf` (`gates-station/`) : `test-fast` 0 et `gate-ci` 0 (ils ont lu un arbre déjà fusionné avec `main` en cours de route).

### Gates

- Local, au dernier commit de code (`175a8f2d2`, puis `b4f6847c3` pour `cargo fmt` seulement) : `npm run test:unit` complet **321 fichiers, 5534 tests
  verts** (avant le commit `style`, sans effet sur le frontend) ; `npm run check` 0 erreur ; `lint-i18n-ownership` vert ; `npm run build` vert
  (`front-build.log`) ; `cargo clippy --workspace --all-targets -- -D warnings` vert à `b4f6847c3` (`clippy.log`) ; `cargo fmt --all -- --check` vert à
  `b4f6847c3` ; nextest ciblé `invoice_line_types_e2e` + `update_line_reorder_bumps_version` : 12 tests verts (`vert-T3-T4-T5.log`).
- **Idefix, premier essai (tête `18823753a`)** : `test-fast` **rouge sur `cargo fmt`** (T4 et T5 non formatés, l'oubli de `cargo fmt` est le mien) —
  journal `idefix/20261127-170357-story_49-2-reordonnancement-lignes-test-fast/test-fast.log`. Corrigé au commit `b4f6847c3`.
- **Idefix, second essai (tête `b4f6847c3`)** : `test-fast`, `gate-ci`, `front` lancés ensemble à 18:25 ; **non terminés au moment d'écrire** (file d'attente
  du verrou d'idefix derrière les gates d'autres stories) — journaux attendus dans `idefix/20261127-182534-…-test-fast`, `…182540-…-gate-ci`,
  `…182546-…-front`. **Non lancés : l'E2E complet** (dû à la clôture).

### File List

- `frontend/src/lib/features/invoices/invoice-lines.ts` (`deplacerLigne`)
- `frontend/src/lib/features/invoices/invoice-lines.test.ts` (T1)
- `frontend/src/lib/components/invoices/InvoiceForm.svelte` (snippet `actionsDeLigne`, `moveLine`, zone `aria-live`)
- `frontend/src/lib/components/invoices/InvoiceForm.test.ts` (T2)
- `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal` 3963, T7)
- `frontend/tests/e2e/invoice-line-reorder.spec.ts` (T6, neuf)
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` (trois clés)
- `crates/kesh-db/src/repositories/invoices.rs` (module de tests seul : T3)
- `crates/kesh-api/tests/invoice_line_types_e2e.rs` (T4, T5, en-tête)
- `docs/manual/fr/user-manual.tex`, `docs/api-external.md`, `README.md`
- `_bmad-output/implementation-artifacts/49-2-reordonnancement-lignes.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- 2026-11-27 — Fiche créée (Opus 5.5, `bmad-create-story`) sur `0e99cd2db` : D1 à D12, AC1 à AC14, T1 à T8 ; aucune
  migration ; deux modules de production (frontend `invoices`, `kesh-i18n`), pas de découpage ; glisser-déposer écarté
  (D3). Validation P1 à lancer (Sonnet), prompt `49-2-reordonnancement-lignes-validation-prompt-p1.md`.
- 2026-11-27 — **Validation P1** (Sonnet 5.5, passe complète, contexte frais ; `kesh-gate-logs/e492/validation-49-2-p1.md`) : **0/0/2/7** — V1-1 (MEDIUM, manuel : la dévalidation n'est pas un moyen de réordonner une facture remise), V1-2 (MEDIUM, AC11 : la commande ne prouve pas « aucun code de production »), V1-3 à V1-9 (LOW : aides de T4, S2 non clos, citation d'erreur, focus jsdom, zone `aria-live`, README, issue #1242). Les neuf sont portés à la fiche ; passe P2 (Opus) à lancer, prompt `49-2-reordonnancement-lignes-validation-prompt-p2.md`.
- 2026-11-27 — **Validation P2** (Opus 5.5, passe complète, contexte frais ; `kesh-gate-logs/e492/validation-49-2-p2.md`) : **0/0/0/5**, les neuf corrections de la P1 relues au code et tenues. V2-1 (LOW, renvoi du manuel : encadré `:2166-2172`, sous-section `:2091` sans `\label`), V2-2 (LOW, T4/T5 : aide `banque` pour le PDF), V2-3 (LOW, D7 : messages sans ligne effacés aussi, accepté), V2-4 (LOW, D5 : icônes `ArrowUp`/`ArrowDown`), V2-5 (LOW, où s'écrit le `closes #1756`), tous portés à la fiche. Décisions du Project Lead du 2026-11-27 portées : #1756 fermée par la story (PR vers `main`), #1242 en `refs` seulement, glisser-déposer → #1757 (« Plus tard »), page de modification d'une facture validée → #1758 (P3), plafond de la story de solde = deux issues. **Tendance** : P1 (Sonnet) 0/0/2/7 → P2 (Opus) 0/0/0/5 ; aucun reclassement. Boucle de validation **terminée** sur une passe sans finding au-dessus de LOW : **ready-for-dev**.
- 2026-11-27 — **Développement** (Opus 5.5, `bmad-dev-story`) : `deplacerLigne` (T1), boutons ↑ / ↓ et zone `aria-live` dans `InvoiceForm.svelte` (T2),
  trois clés × quatre locales et `sitesTotal` 3963 (T7), T3 à T5 côté Rust (tests seuls, aucune ligne de production hors module de tests, AC11),
  E2E T6, manuel, guide de l'API, README ; captures C1 à C6. Écarts à la fiche : label du manuel déjà présent (`sec:suppression-facture`) ;
  deux mutations Rust remplacées par des équivalents (T3, T4). Statut laissé `in-progress` : les gates d'idefix (`test-fast`, `gate-ci`, `front`)
  n'étaient pas terminés ; le passage en `review` attend leurs journaux.
- 2026-11-27 — **Revue de code P1** (Opus 5.5, `bmad-code-review`, `kesh-gate-logs/e492/49-2-revue-p1.md`) : **0/0/2/11**, plus trois constats antérieurs à la story. **Remédiée** (Opus 5.5) : M1, M2, L1 à L5, L7 à L11 corrigés (L6 : reprise de `main`) ; poubelle verrouillée et libellé en i18n ; l'effacement des messages sans rapport (D7) laissé tel quel. Détail et journaux au Dev Agent Record. Passe P2 (Sonnet) à lancer, prompt `49-2-reordonnancement-lignes-revue-prompt-p2.md`.
- 2026-11-27 — **Revue de code P2** (Sonnet 5.5, `kesh-gate-logs/e492/49-2-revue-p2.md`) : **0/0/1/0** — M-P2-1 : l'E2E appelait `pdftotext`, absent d'idefix et de tout prérequis documenté. **Remédié par l'orchestrateur** : bloc et import retirés, en-tête et titre du test ramenés à ce qu'il prouve, l'ordre dans le PDF renvoyé à T4. Passe P3 ciblée (Haiku) sur ce seul commit, prompt `49-2-reordonnancement-lignes-revue-prompt-p3.md`.
