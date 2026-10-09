# Story 15.1a-ii : Les gardes du lettrage — gel des écritures lettrées, contre-passation qui lettre

## Status

done *(revue de code close le 2026-10-09, P2 ciblée à 0 MEDIUM ; développée le 2026-10-09 sur `0724904c` ; créée le 2026-10-09 au découpage de la 15-1a en validation P3 — C124 ; corps repris de
`15-1a-socle-lettrage.md` (validations P1 et P2 remédiées, P3 remédiée ici) ; validations P4 et **P5
remédiées le 2026-10-09 — passe P6 à lancer** avant tout développement ; prérequis : **15-1a-i mergée**, et, comme elle, la 15-12a ; la 15-12b de
préférence avant)*

## Story

**As a** indépendant, PME ou fiduciaire qui tient ses comptes dans Kesh,
**I want** que la marque de lettrage survive à tout ce que l'application fait par ailleurs aux
écritures,
**so that** ce que le logiciel affirme sur ce qui reste ouvert reste vrai le lendemain.

Seconde moitié du **socle** du lettrage (issue **#518**, P1 ; FR85, FR86), issue du découpage de la
15-1a (C124, couture écrite à C118) : **les gardes** — une écriture dont une ligne est lettrée ne se
modifie ni ne se supprime (`ENTRY_LETTERED`, sur les trois chemins : `PUT`, `DELETE` et la dévalidation,
motif d'écran du `GET`), l'écran de la fiche d'écriture le dit, et la contre-passation lettre ce qui
est libre (groupe `reversal`). La première moitié, **`15-1a-i-marque-du-lettrage.md`**, pose la marque :
schéma, primitive, routes.

Ordre de développement : **15-12a → 15-12b → 15-1a-i → 15-1a-ii → 15-1a2 → 15-1b → 15-1c**.

⚠️ **Numérotation conservée** (C124) : cette fiche porte R6, AC8, AC9 et la part (ii) d'AC10, d'AC13 et
d'AC15, et les tâches T4, T4-bis, T5 et la part (ii) de T0, T10, T11 et T12 ; le reste (R1–R5, R7,
AC1–AC7, AC10–AC14, part i) est dans la 15-1a-i. Table de correspondance : `15-1a-socle-lettrage.md`.

## Ce que cette fiche reçoit de la 15-1a-i — dépendance, écrite

Livrable et testable **une fois la 15-1a-i mergée**, dont elle consomme : les colonnes
`lettering_key`/`lettering_origin` et leurs index (R1) ; la primitive `letterings::create_group_in_tx` en
mode `System { held_open_fiscal_year_id }`, origine `Reversal`, sans la cause 5 d'AC3, qui lit sans
verrou le nom des exercices pour l'audit (R3, R7 point 3, C128) ; `letterings::is_letterable_account` (R4) ;
`dissolve_group_in_tx` et la route `DELETE /letterings/{key}` (le geste que prescrit `ENTRY_LETTERED`) ;
`JournalEntryLineResponse` et ses trois champs (AC14), que lit le test du corps `201` (AC9 (a)) ;
`code_from_key` (R2) ; la règle des périodes (R7), dont dépend la précédence d'AC8 ; le test
`lettering_invariants` (AC13), qu'elle étend. **Entre les deux merges, la fenêtre** : une écriture
manuelle lettrée par l'API est modifiable et supprimable — d'où la règle de publication de C124 : la
v0.13.0 ne se tague pas avant que cette fiche soit mergée.

## Reçu de la 15-1a-i — revue de code P2 (2026-10-09) : textes provisoires à retirer

*Section ajoutée par la remédiation de la revue de code P2 de la 15-1a-i (finding A2-1, registre
C-15-1a-i-7 et C-15-1a-i-11). Elle ne réécrit pas cette fiche : elle transmet ce que la 15-1a-i a écrit
au présent de son code livré, et que **cette** story rend faux.*

La revue de code P1 de la 15-1a-i a ramené les textes publics à ce que fait son code : seul le lettrage
`manual` existe, les origines `reversal` et `document` sont « réservées ». Les numéros de ligne sont ceux de la branche `story/15-1a-i-marque-du-lettrage` au commit de sa revue de
code P2 ; ils bougeront au merge — **re-greper par la valeur**. Relevé de C-15-1a-i-7, tel qu'écrit au
registre : `git grep -nE "Seul le lettrage manuel|ne lettre (pas encore|rien|encore rien)|réservé|n'existe encore" -- CHANGELOG.md docs` (62 lignes, dont 56 hors sujet — « réservé à l'administrateur » pour la plupart —, à
trier). Forme resserrée, qui rend **exactement les six sites** ci-dessous sur la branche de la 15-1a-i :
`git grep -nE "Seul le lettrage manuel|ne lettre (pas encore|rien|encore rien)|sont réservés aux lettrages|n'existe encore" -- CHANGELOG.md docs website README.md` ; et le PDF aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "Kesh ne lettre encore rien[^.]*\."`.

| site | texte provisoire |
|---|---|
| `CHANGELOG.md:15` | « **Seul le lettrage manuel existe à ce stade** : Kesh ne lettre pas encore de lui-même une facture soldée par ses règlements, ni une écriture et sa contre-passation. » |
| `docs/api-external.md:222` | `letteringOrigin` (« `manual` ; `document` et `reversal` sont réservés aux lettrages que Kesh posera de lui-même ») |
| `docs/api-external.md:288` | « L'origine d'un groupe est `manual` … ⚠️ **À ce stade, Kesh ne lettre rien de lui-même** : les origines `reversal` … et `document` … sont réservées … ; aucune route ne les rend encore. » |
| `docs/api-external.md:321` | refus `LETTERING_IS_DOCUMENT` du `DELETE`, annoté « *(aucun groupe `document` n'existe encore)* » |
| `docs/api-external.md:322` | refus `LETTERING_LINE_OWNED_BY_DOCUMENT` du `DELETE`, annoté « *(aucun groupe `reversal` n'existe encore)* » |
| `docs/manual/fr/user-manual.tex:2405-2406` + PDF | glossaire, entrée *Lettrage* : « Kesh ne lettre encore rien de lui-même~: ni une facture soldée par ses règlements, ni une écriture et sa contre-passation. » |

⛔ **Règle de relais** : chaque story réécrit ces textes **dans le même commit** que le comportement qui
les rend faux — non dans un commit de documentation ultérieur. Aucun test ne lit ces fichiers : rien ne
rougira si l'un d'eux est oublié, et C124 interdit tout tag entre les merges, si bien qu'un texte oublié
part dans la v0.13.0.

**Ce qui revient à cette story** (la contre-passation qui lettre, R6) : CHANGELOG `:15` — retirer « ni une
écriture et sa contre-passation » et réécrire « Seul le lettrage manuel existe à ce stade » (la
contre-passation lettre désormais d'elle-même) ; `api-external.md:222` et `:288` — sortir `reversal` de la
réserve, et dire que la contre-passation rend des groupes `reversal` ; `:322` — retirer l'annotation
« (aucun groupe `reversal` n'existe encore) » ; glossaire `.tex` `:2405-2406` et PDF régénéré — retirer le
second « ni ». Les sites de `document` (`:222`, `:288`, `:321`, le premier « ni ») restent à la 15-1a2.

⚠️ **Contradiction avec AC15 (ii) si ces sites sont omis.** AC15 (ii) prescrit d'ajouter au site qui
nomme `POST /journal-entries/{id}/reverse` (`:255` au `5e4bec50` de son tableau, `:260` sur la branche
de la 15-1a-i) : « La contre-passation **lettre** ce qui est libre … forme avec son miroir un groupe
`reversal` ». Cette phrase contredit directement `:288` (« Kesh ne lettre rien de lui-même »), `:322`, le
CHANGELOG `:15` et le glossaire — aucun des six sites ne figure au tableau d'AC15 (ii) ni à T11 (part ii).
Ils s'y ajoutent, au même commit que R6.

## Décisions

### R6 — La contre-passation ne défait jamais un groupe ; elle lettre ce qui est libre (C97)

*(Décision reprise de la 15-1a sans changement de fond en validation P3 ; elle consomme la primitive de la 15-1a-i en mode `System`.)*

Dans `reverse_in_tx_inner` (`journal_entries.rs:2175`), **après** l'insertion de l'écriture
inverse, pour chaque ligne `L` de l'origine sur un compte lettrable :

- `L` **non lettrée** → groupe `{L, L'}` d'origine `reversal` (L' = sa ligne miroir, de même
  **position** dans l'ordre `ORDER BY line_order` — `create_in_tx_inner` renumérote `idx + 1`,
  `:446-447`, si bien qu'une origine aux `line_order` non contigus garde l'appariement par rang ;
  sens inversé, même montant : la somme est nulle par construction) ;
- `L` **déjà lettrée** → son groupe **reste tel quel**, et `L'` **reste ouverte**.

**Pourquoi la seconde branche est juste, et non un pis-aller** : le groupe dit qu'à sa date, `L`
était soldée par ses partenaires — c'est vrai et c'est du passé, éventuellement clos. La
contre-passation est un **mouvement nouveau**, daté du jour : `L'` est exactement la créance (ou
la dette) qu'elle fait renaître. La somme des lignes ouvertes reste égale au solde (invariant de
la 15-1b). C'est aussi la seule conduite qui ne bute jamais sur un exercice clos — la
contre-passation générique **accepte** une origine sur exercice clos (elle se date dans
l'exercice ouvert du jour, `find_open_covering_date`, `fiscal_years.rs:550-555`, `FOR UPDATE`).

**Comment R6 lit l'état des lignes** (sans contrôle de flux par l'erreur) : la requête actuelle
de l'origine (`journal_entries.rs:2266`, `SELECT account_id, debit, credit, project_id … ORDER BY
line_order`) ne lit ni `id` ni `lettering_key` et n'est pas verrouillante. Elle est **étendue** :
`SELECT id, account_id, debit, credit, project_id, lettering_key … WHERE entry_id = ? ORDER BY
line_order FOR UPDATE` — l'en-tête de l'origine est déjà tenu (étape (1) de la fonction), l'ordre
en-tête → lignes est respecté. Le test « déjà lettrée » se fait sur cette lecture, **avant**
l'appel à `create_group_in_tx` ; la lettrabilité du compte aussi, **par la fonction de la 15-1a-i**,
`letterings::is_letterable_account` (R4 — la même que la primitive et la 15-1b ; pas de seconde
définition, qui divergerait : validation P4 — F-9). L'appel se fait en mode
`System { held_open_fiscal_year_id }` = l'exercice du jour, déjà verrouillé : il couvre `L'`.

⛔ **Ce que « l'origine reste intacte » veut dire désormais** *(validation P4 — F-3, C129)*. R6 écrit
sur les lignes de l'**origine** (`lettering_key`, `lettering_origin`), sans toucher à son en-tête ni à
sa `version`. Décision : la contre-passation laisse l'origine **intacte dans ses montants, ses
comptes, ses dates et ses libellés** — ce qu'exige l'art. 958f CO, la correction apparente et non
substituée — ; **seules ses lignes reçoivent la marque de lettrage** qui les apparie à la
contre-passation. La doctrine « crée une écriture, n'en modifie aucune » est écrite, sans cette
réserve, à **cinq** endroits que **cette story** réécrit dans ce sens *(trois jusqu'en P5 : le
manuel utilisateur manquait — validation P5, F5-1 ; quatre jusqu'en P6 : la doc frontend de
`reverseJournalEntry` manquait — validation P6, R6-2)*, plus les textes du test renommé (ci-dessous) :

| site (au 2026-10-09, `5e4bec50` = `ec745d0c` pour ces fichiers) | geste |
|---|---|
| `docs/manual/fr/user-manual.tex:608`, § « Corriger une écriture : la contre-passation » (« \textbf{L'écriture d'origine n'est pas touchée} : les deux restent visibles au grand livre ») — la phrase **principale** du manuel sur la contre-passation, en gras ; elle ne dit nulle part que la contre-passation **lettre** *(validation P5 — F5-1)* | « \textbf{L'écriture d'origine n'est pas modifiée} --- ni ses montants, ni ses comptes, ni sa date, ni son libellé~: les deux restent visibles au grand livre, et chacune renvoie à l'autre depuis sa fiche. Seule marque posée sur l'origine~: ses lignes sur un compte lettrable encore ouvertes sont \textbf{lettrées} avec leur miroir dans l'écriture inverse (groupe «~contre-passation~») --- la ligne et sa correction se soldent l'une l'autre~; une ligne déjà lettrée garde son groupe, et son miroir reste ouvert. » Le reste du paragraphe (art.~958f CO, « la correction soit apparente ») est inchangé. |
| doc-comment de `reverse_journal_entry`, `crates/kesh-api/src/routes/journal_entries.rs:477-479` (« ⛔ **Crée une écriture, n'en modifie aucune.** L'origine reste intacte ») | « ⛔ **Crée une écriture ; de l'origine, ne touche qu'à la marque de lettrage.** L'origine reste intacte dans ses montants, comptes, dates et libellés : c'est l'exigence de l'art. 958f CO — la correction doit être apparente, non substituée à ce qu'elle corrige. Seules ses lignes lettrables encore ouvertes reçoivent la marque `reversal` qui les apparie à leur miroir (R6, Story 15-1a-ii). » |
| clé `journal-entries-reverse-dialog-body` — `fr-CH:357`, `de-CH:363`, `en-CH:363`, `it-CH:363` — lue à **chaque** contre-passation | texte ci-dessous, quatre locales |
| repli `frontend/src/routes/(app)/journal-entries/[id]/+page.svelte:475` | identique au catalogue FR |
| doc de `reverseJournalEntry`, `frontend/src/lib/features/journal-entries/journal-entries.api.ts:42-45` (« ⛔ Ne modifie rien : crée l'écriture inverse et rend celle-ci. L'origine demeure ») — jumeau frontend du doc-comment de la route *(validation P6, R6-2)* | « ⛔ Crée l'écriture inverse et rend celle-ci ; de l'origine, ne touche qu'à la marque de lettrage (ses lignes lettrables encore ouvertes sont lettrées avec leur miroir, groupe `reversal` — Story 15-1a-ii). L'origine demeure dans ses montants, comptes, date et libellé — c'est la correction qui doit se voir, pas remplacer ce qu'elle corrige. » Le fichier est déjà touché (doc de `deleteJournalEntry`, T4-bis). |

| locale | `journal-entries-reverse-dialog-body` |
|---|---|
| fr-CH | Kesh créera une écriture inverse à la date du jour. L'écriture d'origine reste intacte dans ses montants, ses comptes, sa date et son libellé — seules ses lignes reçoivent, le cas échéant, la marque de lettrage qui les apparie à l'écriture inverse : c'est la correction qui doit se voir, pas disparaître. |
| de-CH | Kesh erstellt eine Gegenbuchung mit dem heutigen Datum. Die ursprüngliche Buchung bleibt in Beträgen, Konten, Datum und Text unverändert — nur ihre Zeilen erhalten gegebenenfalls die Ausgleichsmarke, die sie mit der Gegenbuchung verbindet: Die Korrektur muss sichtbar sein, nicht verschwinden. |
| en-CH | Kesh will create an opposite entry dated today. The original entry stays untouched in its amounts, accounts, date and description — only its lines receive, where applicable, the matching mark that pairs them with the opposite entry: a correction must be visible, not disappear. |
| it-CH | Kesh creerà una scrittura inversa in data odierna. La scrittura originale resta intatta negli importi, nei conti, nella data e nella descrizione — solo le sue righe ricevono, se del caso, il segno di abbinamento che le accoppia alla scrittura inversa: la correzione deve vedersi, non sparire. |

⚠️ *Terminologie* : « Ausgleichsmarke », « matching mark », « segno di abbinamento » sont à aligner sur
les traductions que la 15-1a-i aura retenues pour ses dix clés (T10 de la 15-1a-i, qui ne fixe que le
FR) — une seule désignation du lettrage par langue.

**Trié, non touché** : `CHANGELOG.md:192` (« l'écriture inverse laisse l'origine intacte ») est sous
`## [0.12.0]`, **publié** ; de même `README.md:220`, ligne **v0.12.0** de la feuille de route (« l'écriture
inverse laissant l'origine intacte »), historique d'une version publiée *(validation P5 — L-4)* ;
`user-manual.tex:2226` (« Kesh crée l'écriture inverse, garde l'origine,
et lie les deux ») reste exact ; `user-manual.tex:1230`, `:1235` (l'avoir laisse l'écriture d'origine
intacte) relèvent du groupe `document` de la 15-1a2, à qui la même question est portée (« Reçu de la
15-1a », point 20) ; `admin-manual.tex:2128` (« laisse l'écriture d'origine en place ») et
`user-manual.tex:1164`, `:1399` (« restent toutes deux au grand livre ») restent vrais. Relevé —
**par la valeur**, la liste de P4 ayant manqué « n'est pas touchée » et « ne touche pas »
(validation P5, F5-1, R5-6 ; « ne modifie rien » et « demeure » ajoutés en validation P6, R6-2) :
`git grep -nE "n'en modifie aucune|ne modifie rien|n.est pas touchée|ne touche pas|pas
être touchée|inchangée|intactes?|demeure|garde l'origine|untouched|unverändert|resta intatta" origin/main --
crates frontend/src docs CHANGELOG.md README.md`, puis tri à la main (le motif rend de nombreux
« intacte » étrangers à la contre-passation — import, contacts, factures — et « demeure » rend les
« mises en demeure » des rappels ; sur `ec745d0c`, le seul « demeure » qui parle de la contre-passation
est `journal-entries.api.ts:44`). Sur les PDF aplatis,
greper `pas touchée` **sans** l'apostrophe (typographique dans le PDF).

Le test existant `reverse_creates_the_opposite_entry_and_leaves_the_origin_intact`
(`crates/kesh-api/tests/journal_entry_reversal_e2e.rs:263`) emploie un compte `1020` `Asset` sans
`bank_accounts` — **lettrable** : après R6, sa ligne 1020 de l'origine est lettrée, et le test, qui ne
relit que `account_id, debit, credit` (`:306-319`), resterait vert en disant « les lignes de l'origine
sont intactes ». Il est **renommé** `reverse_creates_the_opposite_entry_and_marks_the_origin_without_altering_it`
et **étendu** (T12) : montants et comptes de l'origine inchangés **et** `version` inchangée (déjà
asserté), **et** la marque posée — la ligne 1020 de l'origine porte une `lettering_key` non nulle,
égale à celle de sa ligne miroir, origine `reversal` ; la ligne 6000 (`Expense`) n'en porte pas.
⚠️ **Ses textes changent avec son nom** *(validation P5 — R5-6 = L-3 : ils disent le contraire du nom
neuf)* : docstring `:257-258` (« la contre-passation crée l'inverse et **ne touche pas** l'origine »),
commentaire `:281-283` (« L'origine est inchangée — les SIX champs »), message `:303` (« l'écriture
d'origine ne doit pas être touchée ») et message `:318` (« les lignes de l'origine sont intactes ») →
« inchangée dans ses montants, comptes, date et libellé ; seule la marque de lettrage est posée » (ou
la forme la plus courte qui le dise, au site d'un message d'assertion).

⚠️ **Les annulations de PIÈCE ne relèvent pas de la seconde branche** : elles dissolvent le
groupe `document` **avant** d'appeler le socle (15-1a2), si bien que la ligne annulée est libre
et se lettre avec son miroir. Ce socle n'en sait rien et n'a pas à le savoir.

⚠️ Le lettrage `reversal` passe par `create_group_in_tx` — **audité** (AC10 de la 15-1a-i, part ii ici), sous les verrous
déjà tenus par le socle.

⛔ **La valeur rendue est relue après le lettrage** *(validation P2 — F-4, C116)*. Aujourd'hui
`reverse_in_tx_inner` rend `created`, lu par `create_in_tx_inner` à son étape 7 (`SELECT
{LINE_COLUMNS}`, `journal_entries.rs:492` ; retour `:516`) **avant** tout lettrage, et le renvoie
tel quel après l'audit (`:2349`) ; la route en fait son `201` (`routes/journal_entries.rs:490-497`,
`JournalEntryResponse::from(created)`). Sans relecture, la réponse dirait `letteringCode: null` sur
une ligne que la base porte lettrée — l'API mentirait à l'instant même du geste. Donc : **si au moins
un groupe a été posé**, `created.lines` est remplacé par une relecture `SELECT {LINE_COLUMNS} FROM
journal_entry_lines WHERE entry_id = ? ORDER BY line_order` de l'écriture inverse, dans la
transaction, avant le retour. La correction vit dans `reverse_in_tx_inner`, pas dans la route : tous
les appelants de `reverse_in_tx` (annulations de règlement, de facture fournisseur, dé-rapprochement)
reçoivent ainsi une valeur exacte. Test qui **lit le corps du `201`** (AC9 (a)).

⚠️ **Coût, écrit d'avance** *(F-12)* : chaque ligne lettrable de l'origine fait un
`create_group_in_tx` complet (lecture verrouillante jointe, lettrabilité, lecture **ordinaire** du nom
des exercices du groupe — R7 point 3 de la 15-1a-i, C128 —, `UPDATE`, audit) sous le verrou de
l'exercice du jour — une écriture d'ouverture d'une soixantaine de lignes de bilan coûte de l'ordre
de 360 requêtes (300 jusqu'en P4 ; une lecture de plus par groupe). Acceptable à l'échelle de Kesh ; si une mesure le contredit, la
lettrabilité des comptes se lit **une fois** pour toutes les lignes avant la boucle (lecture, pas
écriture) — l'écriture de la marque, elle, reste dans la seule primitive (R3, 15-1a-i).

⚠️ **Un groupe `reversal` dont une ligne appartient à une PIÈCE ne se dissout pas à la main**
*(C106)* : sinon l'utilisateur ouvrirait pour toujours une paire que R5 (15-1a-i) lui interdit de relettrer.
Refus nommé à AC5 (point 2) de la 15-1a-i.

## Critères d'acceptation

**AC8 — Gel des lignes lettrées face à la modification et à la suppression** *(réécrit en
validation P1 — C102 ; rang révisé en validation P3 — R3-1 = F3-1, C126)*. Une écriture dont **une**
ligne porte une marque est refusée en **409 `ENTRY_LETTERED`**, message « Cette écriture est lettrée :
délettrez-la d’abord. » (apostrophe typographique, comme les clés voisines — validation P5, L-8) suffixé du code du premier groupe (ordre `id`) entre parenthèses — la forme
commune à tous les motifs de gel (`entry_document_refusal_response`, `format!("{base} ({numero})")`) :
la parenthèse suit le point final comme pour un numéro de pièce, cosmétique assumé (F-15 de P2).
⚠️ **Le message prescrit un geste que l'écran n'offre pas encore** *(validation P4 — R4-9 = F-8)* : avant
la 15-1c, le délettrage ne se fait que par l'API (`DELETE /api/v1/letterings/{key}`), ce que le manuel
dit (sixième condition, AC15 part ii) et le message non. Fenêtre étroite — seuls des groupes `manual`
posés par l'API la font voir (R5 exclut les lignes de pièce ; les groupes `reversal` touchent des
écritures que `ENTRY_IS_REVERSED`/`IS_A_REVERSAL` gardent **avant** la marque) — et **jamais publiée** :
l'Epic 15 sort en une release, la v0.13.0 (`README.md:222`), taguée à la clôture de l'epic, donc après
la 15-1c. Le message reste donc tel quel ; il devient exact à la 15-1c. Si la v0.13.0 devait être
taguée avant la 15-1c, la règle de publication de C124 s'étend à elle (à décider par l'orchestrateur).

- **Le motif** est une variante **`ModificationGuard::Lettered { code: String }`**
  (`kesh-db/src/errors.rs:130`) : `code()` = `ENTRY_LETTERED`, `document_id()` = `None`, `label()`
  = `Some(code)`. Il traverse donc tel quel `modification_refusal` (`journal_entries.rs:1058`,
  → `DbError::EntryNotModifiable`) et le mapping `kesh-api/src/errors.rs:2822` (au 2026-10-09,
  `5e4bec50` ; `:2794` jusqu'en P3) — un bras de plus dans le `match` du repli : clé
  `journal-entries-modify-blocked-lettered`. La réponse garde la forme commune
  (`entry_document_refusal_response`, `errors.rs:1145`) : `details.documentId` = `null`,
  `details.documentNumber` = le **code de lettrage** — forme constante pour tous les motifs de gel,
  l'écran suffixe l'étiquette comme pour un numéro de pièce.
- ⛔ **Il est rendu par une fonction DISTINCTE, `lettering_guard(conn, company_id, id, lecture)`,
  et non par `modification_guard`** : `delete_in_tx` n'évalue `modification_guard` que sous
  `enforce_ownership` (`:1719`), et `invoices::unvalidate` l'appelle avec `false`
  (`invoices.rs:1660`). La garde de lettrage est une **étape propre et inconditionnelle**.
- ⛔ **Son rang : APRÈS tout refus que le délettrage ne peut lever** *(C126 — la version de P1/P2 la
  plaçait avant le verrou de période ; depuis C113, délettrer est refusé quand toutes les lignes du
  groupe sont sous le verrou : « délettrez-la d'abord » envoyait alors vers un geste refusé, ou vers un
  délettrage qui réussit puis un `PERIOD_LOCKED` de toute façon — un lettrage juste détruit pour rien.
  C117 avait écarté « la marque avant l'exercice postérieur clos » pour cette raison sans l'appliquer au
  verrou de période)* :
  - `delete_in_tx` : étape **3-quinquies**, **après** 3-quater (le verrou de période), toujours hors du
    drapeau — elle couvre la dévalidation (inatteignable aujourd'hui : une facture lettrée a un
    règlement ou un avoir, que `unvalidate` refuse, `invoices.rs:1530-1607` ; la garde vit quand même
    au point de passage, P3-2 d'août). Le doc-comment « Ordre des refus » (`journal_entries.rs:1593-1610`)
    gagne la ligne **3-quinquies**, et « le verrou de période parle en dernier » (`:1609-1610`, et le
    commentaire de l'étape 3-quater, `:1723-1726`) devient « avant-dernier : la marque de lettrage parle
    après lui » ;
  - `update_in_tx` : étape **7-bis**, **après** l'étape 7 (verrou de période sur l'ancienne date puis
    la nouvelle, `:1419-1432`), **avant** l'instantané et le court-circuit no-op de l'étape 8
    (`:1433`) — les refus du corps (`DATE_OUTSIDE_FISCAL_YEAR`, comptes) et `OPTIMISTIC_LOCK_CONFLICT`
    (étape 4) parlent donc avant elle : un conflit de version ne se lève pas en délettrant (le
    délettrage ne touche pas l'en-tête). Le doc-comment « Ordre des refus » du `PUT` (`update`, `:1248-1258`) la
    nomme après le verrou de période ; le court-circuit **no-op** y vient après toutes les gardes
    (KF-004) : un `PUT` identique sur une écriture lettrée rend donc le refus, pas un `200` ;
  - `modification_blocker` (`:1077`, le `GET` de la fiche) : **après** `PeriodLocked`, dernier motif —
    rendu en `ModificationBlocker::Guard(ModificationGuard::Lettered { .. })` ; son doc-comment
    (`:1068-1071`, « exercice clos, exercice postérieur clos, garde d'écriture, verrou de période ») la
    nomme en dernier ;
  - `invoices::unvalidate` (doc-comment, `invoices.rs:1487`) : la chaîne de précédence que la 15-12b y
    écrit (AC 10 de sa fiche) finit par **`… → PERIOD_LOCKED → ENTRY_LETTERED`** *(L6 de P3)*.
  - **Les doc-comments qui comptent les gardes de `delete_in_tx` tenues hors du drapeau** *(validation
    P4 — R4-2, F-7 ; propagation par la valeur : `grep -nE "3-bis et 3-quater|trois autres|Les trois
    autres empêchements|Celles de .delete_in_tx."`)* : `journal_entries.rs:1640-1641` (section
    « `enforce_ownership` — qui passe quoi » : « Les étapes 3, 3-bis et 3-quater tiennent quand
    même ») gagne **3-quinquies** ; `invoices.rs:1471-1473` (doc d'`unvalidate`, « trois autres y vivent
    déjà (exercice clos, écriture contre-passée, période verrouillée) et parlent après ») garde
    « trois » et gagne « — plus la marque de lettrage (`ModificationGuard::Lettered`), tenue au même
    point de passage mais inatteignable par la dévalidation, qui parle en dernier » ;
    `invoices.rs:1650-1657` (commentaire de l'étape 5, « Celles de
    `delete_in_tx` qui ne dépendent pas du drapeau — exercice clos, contre-passation, période verrouillée
    (#443) — tiennent ») gagne la marque ; `kesh-db/src/errors.rs:241-244` (doc d'`UnvalidationBlocker`,
    « Les trois autres empêchements (exercice clos, contre-passée, période verrouillée) ne sont PAS
    ici ») garde « trois » et nomme de même `ModificationGuard::Lettered`, inatteignable. Le motif vit au
    point de passage : la doc le **nomme** ; elle ne le **compte** pas parmi les refus de la dévalidation.
    ⚠️ **Décompte, tranché une fois** *(validation P5 — R5-7 = L-7, C132 ; P4 écrivait « → quatre »)* :
    les totaux écrits ailleurs comptent les refus **que la dévalidation peut rendre** — « huit » à
    `invoices.rs:1376`, `frontend/src/routes/(app)/invoices/[id]/+page.svelte:355`,
    `admin-manual.tex:1919` (« Les huit refus de la dévalidation ») et `:1961` (« refusée sous huit
    motifs »), et le tableau d'`api-external.md:288-299` (huit refus métier, plus la version et l'état) —
    et restent **tels quels** : un motif inatteignable n'y entre pas. Compter « quatre » ici et « huit »
    là ferait neuf d'un côté et huit de l'autre. ⚠️ **Chiffres relus après la 15-12b** *(validation P6,
    R6-3)* : « trois » et « huit » sont ceux de `ec745d0c`. Dans l'ordre préféré (C112), la 15-12b
    passe la première et rend la dévalidation refusante sous `LATER_FISCAL_YEAR_CLOSED` : elle réécrit
    alors `invoices.rs:1471-1473` et `kesh-db/src/errors.rs:241-244`, et porte les totaux « huit » à
    neuf. Cette fiche ne reprend pas ses chiffres : **la règle** — nommer la marque, ne pas la compter
    parmi les refus de la dévalidation — **vaut quel que soit le total** ; le développeur relit les
    nombres sur le `main` du moment (T0) et ne remet pas « trois » ni « huit » là où la 15-12b a écrit
    juste.

  **Ce que ce rang garantit, et qui justifie le message** : quand `ENTRY_LETTERED` parle sur la route,
  l'écriture est en **période ouverte** (exercice ouvert, aucun postérieur clos, date après la borne :
  R7 de la 15-1a-i) et n'appartient à **aucune pièce** (la garde de la modification a parlé avant) —
  son groupe est donc `manual` (R5) et sa propre ligne satisfait la règle des périodes : le délettrage
  que le message prescrit **aboutit** (hors concurrence). C'est le test
  `entry_lettered_refusal_leads_to_a_dissolution_that_succeeds` (T12).
- ⚠️ **Précédence avec la 15-12b** *(C117, rang renommé par C126)* : la 15-12b rend l'étape 2-bis
  (« exercice postérieur clos », `LaterFiscalYearClosed`) **inconditionnelle** dans `delete_in_tx` ; la
  marque, inconditionnelle aussi, parle désormais **après** 3-quater, donc après 2-bis par
  construction. **La seconde des deux stories à merger** écrit cette précédence au doc-comment « Ordre
  des refus » de `delete_in_tx` et la teste par **une paire** —
  `delete_of_a_lettered_entry_under_a_later_closed_year_says_later_closed` : écriture lettrée dans N,
  N+1 clos (posé par SQL direct), `delete_in_tx(…, false)` **et** `(…, true)` → `LaterFiscalYearClosed`
  dans les deux cas ; mutation tuée : placer la marque avant 2-bis. Dans l'ordre préféré (C112 : 15-12b
  avant 15-1a-ii), c'est **cette fiche** — tâche T4 ; si elle est mergée la première, la paire revient à
  la 15-12b, **réalignée** sur ce découpage par le commit `d0161d96` (étape 3-quinquies et paire,
  `15-12b-filet-sous-un-bilan-clos.md:79-93` ; chaîne `… → PERIOD_LOCKED → ENTRY_LETTERED`, `:126-128`
  — validation P4, R4-6 = F-5 : la mention « nomme encore 3-ter-bis » était périmée ; citation de la
  chaîne corrigée en validation P5, R5-9).
- **`lecture`** : `Lecture::Verrouillante` dans les deux chemins transactionnels (`SELECT id,
  lettering_key FROM journal_entry_lines WHERE entry_id = ? FOR UPDATE`, puis le premier groupe
  par `id` **choisi en Rust**) ; `Lecture::Conseil` dans `modification_blocker` — la **même**
  requête **sans** `FOR UPDATE`. ⚠️ **Ni `ORDER BY id`, ni `LIMIT`, ni filtre `lettering_key IS NOT
  NULL`** *(validation P2 — F-6)* : un `ORDER BY id LIMIT 1` peut inviter l'optimiseur à parcourir
  la clé primaire, et sous `FOR UPDATE` en `REPEATABLE READ` chaque ligne parcourue resterait
  verrouillée — la table entière à chaque `PUT`. Lire toutes les lignes de l'écriture par
  `idx_jel_entry` (une écriture en a peu, et `update_in_tx` / `delete_in_tx` les verrouillent de
  toute façon) ferme ce risque sans dépendre du plan ; `EXPLAIN` en T0 (attendu : `ref` sur
  `idx_jel_entry` ou `uq_jel_entry_order`). `Lecture::Conseil` va sans verrou parce que
  `modification_blocker` lit sur une connexion du pool, hors transaction (« l'écran conseille, le
  `PUT` tranche ») ; une lecture verrouillante y attendrait derrière tout lettrage en cours, jusqu'au
  1205. Une seule requête, un paramètre : pas deux fonctions qui divergeraient.
- **Pourquoi la lecture verrouillante dans les transactions** : depuis la 15-8a, `update_in_tx`
  prend l'en-tête **en premier acte** (`:1307-1309`) et sa vue s'ouvre après ; le premier acte de
  R7 (15-1a-i) tient aussi l'en-tête de toute écriture touchée. Les deux transactions se **sérialisent**
  donc sur l'en-tête — l'une attend l'autre, ou l'une est victime d'un interblocage — ; le `FOR
  UPDATE` des lignes est une défense en profondeur (une lecture courante ne dépend d'aucune vue). ⚠️
  **Mais les deux ordres sont opposés, et un cycle est possible** *(validation P4 — F-4 ; la version
  précédente disait que le verrou d'en-tête « suffit », ce qui laissait croire l'interblocage
  impossible)* : l'acte 1 du lettrage verrouille, ligne par ligne du parcours, la ligne **puis** son
  en-tête ; `update_in_tx`, `delete_in_tx` et `reverse_in_tx_inner` (`:2187`, puis les lignes de
  l'origine avec R6) prennent l'en-tête **puis** les lignes. **La défense est le rejeu**, pas l'ordre :
  les routes de lettrage (AC12 de la 15-1a-i), le `PUT`, le `DELETE` et la contre-passation sont toutes
  `Rejouee` (`routes/journal_entries.rs:490`, `:690`, `:736` — ce dernier, `retry_on_deadlock("journal_entries::delete", …)`,
  ajouté en validation P5, R5-4 ; `audit_route_registry.rs:199/203/205`) — d'où,
  au test « R7 (côté gardes) » de T12, « jamais un 500 ». Cet interblocage est nommé à la première
  puce des « Interblocages résiduels » de R7 (15-1a-i). ⚠️ La garde vient désormais **tard** dans les
  deux chemins (après le verrou de période) : rien ne change à cet argument, l'en-tête étant tenu
  depuis le premier acte.
- ⛔ **Les textes qui décrivent l'ordre des verrous de ces chemins, ou qui disent « aucun cycle
  connu », sont réécrits** *(validation P5 — F5-2 = R5-4 : la story crée un cycle connu sur le
  `DELETE` et en ajoute un au `PUT`, et ces textes, qu'aucun test ne lit, le nieraient en silence)* :

  | site (au 2026-10-09, `5e4bec50` = `ec745d0c`) | ce qu'il dit | geste |
  |---|---|---|
  | `docs/MULTI-TENANT-SCOPING-PATTERNS.md:326` (Pattern 5, ligne du `PUT`) | `journal_entries → [companies → projects] → fiscal_years (…) → (accounts, shared, …)` | séquence complétée : `… → fiscal_years (…) → journal_entry_lines (step 7-bis, FOR UPDATE, Story 15-1a-ii) → (accounts, shared, …)` |
  | `:327` (ligne du `DELETE`) | `journal_entries + fiscal_years (…) → fiscal_years (later years …) → (FK checks of the DELETE …)` | `… → fiscal_years (later years …) → journal_entry_lines (step 3-quinquies, FOR UPDATE, Story 15-1a-ii) → (FK checks …)` |
  | `:331` (note du `PUT`, « Three **inherited** cycles remain ») | trois cycles hérités | un **quatrième**, nommé : « **lines ↔ entry** (Story 15-1a-ii): lettering act 1 locks a line then its entry; the PUT holds the entry, then locks its lines at step 7-bis — replayed » |
  | `:332` (note du `DELETE`) | « **No known cycle.** Mitigation: replayed … **by uniformity with the `PUT`**, not for a known cycle » | « **One known cycle** (Story 15-1a-ii): lettering act 1 (line then entry) ↔ DELETE (entry then lines, step 3-quinquies). Mitigation: replayed (`retry_on_deadlock`, `"journal_entries::delete"`) — the transaction is replayed whole and the deadlock rolled it back. » |
  | doc de `delete_journal_entry`, `crates/kesh-api/src/routes/journal_entries.rs:723-728` | rejouée « par uniformité avec le `PUT`, pas pour un cycle connu : l'ordre de verrous est écriture et exercice (jointure) → exercices postérieurs → contrôles des clés étrangères au `DELETE` » | ordre complété (« → lignes de l'écriture, `FOR UPDATE`, étape 3-quinquies → … ») ; « pas pour un cycle connu » → « contre un cycle connu : l'acte 1 du lettrage prend une ligne puis son écriture, le `DELETE` l'écriture puis ses lignes (Story 15-1a-ii) » |
  | doc du module `crates/kesh-api/tests/audit_route_registry.rs:80-83` (« `journal_entries::delete` (15-8b, rejouée par uniformité avec le `PUT`, sans cycle connu — choix C-15-8b-6) ») et commentaire de l'entrée du `DELETE`, `:200-202` (« Rejouée par la 15-8b (par uniformité avec le `PUT`) ») *(validation P6, R6-1)* | rejeu du `DELETE` justifié par l'uniformité, « sans cycle connu » | `:82` : « … rejouée par uniformité avec le `PUT` à la 15-8b (C-15-8b-6), et contre un cycle connu depuis la 15-1a-ii : l'acte 1 du lettrage prend une ligne puis son écriture, le `DELETE` l'écriture puis ses lignes » ; `:201` : « Rejouée par la 15-8b (par uniformité avec le `PUT`) ; cycle connu avec l'acte 1 du lettrage depuis la Story 15-1a-ii » — l'historique (C-15-8b-6) reste, la négation du cycle sort |
  | doc d'`update`, `kesh-db/src/repositories/journal_entries.rs:1205-1209` (« # Ordre des verrous ») et `:1218-1234` (« ⛔ Cet ordre n'est pas sans cycle — Trois cycles hérités ») | lignes absentes de l'ordre ; trois cycles | « … exercices postérieurs → **lignes de l'écriture** (étape 7-bis, `FOR UPDATE`) → (comptes …) » ; une quatrième puce, **lignes ↔ écriture**, non héritée (Story 15-1a-ii), rejouée |
  | doc de `delete_in_tx`, « # Sérialisation » (`journal_entries.rs:1619-1630`) | premier acte joint, puis exercices postérieurs, puis lectures ordinaires | une phrase : la garde de lettrage (3-quinquies) verrouille ensuite les lignes de l'écriture — ordre écriture → lignes, opposé à l'acte 1 du lettrage, rejoué |
  | lecture étendue des lignes de l'origine dans `reverse_in_tx_inner` (R6, `journal_entries.rs:2266`) | — (verrou neuf) | un commentaire au site : ordre en-tête de l'origine → ses lignes, opposé à l'acte 1 du lettrage, défense par le rejeu de la contre-passation et des annulations qui l'appellent. **Pas de ligne Pattern 5** : la contre-passation n'y en a aucune (`grep -n "reverse" docs/MULTI-TENANT-SCOPING-PATTERNS.md` → `:400`, un exemple de code), et documenter tout son ordre de verrous est hors du périmètre — même tranchage qu'`api-external.md` (C131, point 6) |

  Le texte anglais des cellules est indicatif (Pattern 5 est en anglais) ; ce qui est prescrit, c'est
  que le cycle soit **nommé** et l'ordre **complet**.

  **Relevé, par la valeur** *(validation P6, R6-1 : la liste de P5 était partie de la formulation de
  Pattern 5 et de la route, et manquait les deux sites d'`audit_route_registry.rs`)* :
  `git grep -nE "sans cycle|cycle connu|par uniformité|No known cycle|by uniformity|known cycle" origin/main
  -- crates frontend/src docs README.md CHANGELOG.md website`. Sur `ec745d0c`, il rend huit lignes,
  toutes triées : `routes/journal_entries.rs:724`, `:725` (inscrites, `:723-728`) ;
  `audit_route_registry.rs:82`, `:201` (inscrites, ligne ci-dessus) ; `journal_entries.rs:1218`
  (inscrite, `:1218-1234`) ; `MULTI-TENANT-SCOPING-PATTERNS.md:332` (inscrite) ;
  **étrangères** : `audit_route_registry.rs:95` (« sans cycle démontré », candidates
  `SansEcritureAuJournal` de l'exception (iv) — factures, projets, clôture —, aucune n'est le `DELETE`
  d'une écriture) et `opening_complement.rs:26` (règle de verrouillage du complément d'ouverture ; son
  exemple « la contre-passation l'écriture puis les exercices » reste vrai). Le développeur rejoue ce
  relevé sur le `main` du moment (T0) ; tout site neuf est inscrit ou trié.

**L'écran existant est touché** (le motif sort par `GET /journal-entries/{id}`,
`kesh-api/src/routes/journal_entries.rs:461`, champ `modificationBlockedBy`) — sans quoi la fiche
masquerait « Modifier » / « Supprimer » avec un motif **vide** (`default` du `switch` → `''`) et un
`PUT` refusé tomberait en `'other'` :

- `frontend/src/lib/features/journal-entries/journal-entries.types.ts:91` : `ModificationBlocker`
  gagne `'ENTRY_LETTERED'` ; doc-comment `:85` « les onze codes » → « les douze » ;
- `blocker-messages.ts:81` (`modificationBlockerLabel`) : une branche, clé
  `journal-entries-modify-blocked-lettered`, repli FR identique au catalogue ; **pas** dans
  `MODIFICATION_LABEL_IN_MESSAGE` (la fiche suffixe le code comme un numéro de pièce) ;
- `form-helpers.ts:108-130` (`editRefusalOutcome`) : `'ENTRY_LETTERED'` → `'stale'` ;
- tests : `blocker-messages.test.ts:25` (`ATTENDU` gagne la ligne ; commentaire `:24` « Les onze codes
  d'écran » → douze ; titre `:50` « couvre exactement onze codes » → douze ; `toHaveLength(11)` `:52`
  **et** `expect(new Set(messages).size).toBe(11)` `:53` → `12` — validation P4, R4-7), `form-helpers.test.ts` (le code rend `'stale'`),
  `JournalEntryForm.edit.test.ts:132` (`it.each` des codes rendus `'stale'` côté formulaire :
  `'ENTRY_LETTERED'` s'y ajoute) ;
- commentaire `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts:160-165` *(L9 de P3 — seul « les
  onze codes » était nommé)* : « les onze codes de modification — **quatre** branches propres
  déléguant à `i18nMsg`, sept qui délèguent à `reversalBlockerLabel` » devient « les **douze** codes —
  **cinq** branches propres …, sept … » (la branche `ENTRY_LETTERED` est propre) ; les compteurs du
  relevé (`ecartee`, `conforme`) ne bougent pas — la fonction est déjà comptée ;
- `kesh-db/src/errors.rs:206` : « l'une des onze valeurs » → « douze » *(citée `:201` jusqu'en P2 ;
  `:201` est la doc de `PeriodLocked` — R2-4)* ;
- `kesh-db/src/errors.rs`, trois doc-comments que `Lettered` rend faux *(validation P4 — R4-2)* :
  `:121-122` (doc de `ModificationGuard` : « C'est la **garde d'écriture** : rendue par
  `journal_entries::modification_guard` ») — la variante `Lettered` est rendue par `lettering_guard`,
  fonction distincte, le dire ; `:189-191` (doc de `ModificationBlocker`, « exercice clos, exercice
  postérieur clos, garde d'écriture, verrou de période (ancienne date) ») — la marque de lettrage, rendue
  en `Guard(Lettered)`, parle **après** le verrou de période, l'écrire ; `:199` (doc de la variante
  `Guard`, « Garde d'écriture (pièce, contre-passation, paiement détaché) ») — « … ou marque de
  lettrage, qui parle en dernier » ;
- `kesh-db/src/errors.rs`, les doc-comments des **méthodes** que `Lettered` rend faux *(validation P5 —
  R5-3 = L-2 ; propagation par la valeur : `grep -nE "Code canonique exposé|Identifiant de la
  pièce|Étiquette lisible de la pièce|Étiquette : nom de l'exercice"`)* : `:152-153` (`code()` : « celui
  du motif de contre-passation, ou `DETACHED_SUPPLIER_SETTLEMENT` ») → « …, `DETACHED_SUPPLIER_SETTLEMENT`,
  ou `ENTRY_LETTERED` » ; `:161` (`document_id()` : « Identifiant de la pièce … ») → « …, `None` pour la
  marque de lettrage » ; `:172` (`label()` : « Étiquette lisible de la pièce (numéro de facture,
  d'avoir…) ») → « … ou code de lettrage » ; `:216-217` (`ModificationBlocker::label` : « nom de
  l'exercice postérieur clos, numéro de pièce, ou borne du verrou ») → « … ou code de lettrage » ; et,
  côté écran, `frontend/src/lib/features/journal-entries/blocker-messages.ts:76-79` (doc de
  `modificationBlockerLabel`, même énumération) → « … ou le code de lettrage » ;
- **les doc-comments qui énumèrent le cadre de la modification ou de la suppression** *(validation P5 —
  R5-5 = L-1 ; propagation par la valeur : `git grep -nE "aucune pièce|pas un paiement détaché|tant que
  l.exercice est ouvert, par modification" -- crates/*/src frontend/src`)* — chacun gagne « aucune
  ligne lettrée », **en dernier** : doc de `delete_journal_entry`, `routes/journal_entries.rs:711-714` ;
  doc du module `repositories/journal_entries.rs`, `:25-27` (« une erreur se corrige de préférence par
  contre-passation — ou, tant que l'exercice est ouvert, par modification tracée ») et `:44-48`
  (« # Modification ») ; doc d'`update`, `:1161-1164` ; doc de `delete_by_id`, `:1552-1555` (celle
  « qu'on ouvre en premier ») ; doc de `deleteJournalEntry`,
  `frontend/src/lib/features/journal-entries/journal-entries.api.ts:69-71` ;
- doc de `modification_blocker`, `journal_entries.rs:1073` : « cinq lectures enchaînées » → **six**
  (`lettering_guard`, `Lecture::Conseil`, en ajoute une — validation P5, R5-8) ;
- `kesh-api/src/routes/journal_entries.rs:163-166` (au 2026-10-09 ; `:166-168` jusqu'en P3 — L3) :
  doc-comment de `modification_blocked_by` (« l'une de onze valeurs », et l'énumération) → douze,
  `ENTRY_LETTERED` nommé en dernier *(R2-6)* ; **et** `:168-169`, doc de `modification_blocked_label`
  (« Numéro de pièce, nom de l'exercice postérieur clos, ou borne du verrou ») gagne « ou code de
  lettrage » *(validation P4 — R4-3)* ; de même la doc de `modificationBlockedLabel` dans
  `frontend/src/lib/features/journal-entries/journal-entries.types.ts:77-80` ;
- **le test serveur de la table de correspondance** *(validation P4 — F-1 : il garde « les onze codes
  d'écran » et resterait vert sans exercer le douzième — test muet)* :
  `each_screen_code_maps_to_its_put_and_delete_refusal`
  (`crates/kesh-api/tests/journal_entry_reversal_e2e.rs:2108`, attribut `:2107` ; `:2110` jusqu'en P5 —
  R5-9) gagne le cas `ENTRY_LETTERED` — écran
  `ENTRY_LETTERED` ↔ `PUT` `409 ENTRY_LETTERED` ↔ `DELETE` `409 ENTRY_LETTERED`, montage qui ne porte
  QUE cette cause (une écriture manuelle en période ouverte, lettrée avec une autre) — ; `vus.len()`
  `11` → `12` (`:2218`), message `:2219` et docstring `:2102` « onze » → « douze ». **Mutation tuée** :
  retirer l'appel de `lettering_guard` de `modification_blocker` (l'écran ne rend plus le code, `vus`
  reste à 11) ;
- `docs/api-external.md` : sites d'AC15 (part ii).

⚠️ **Grep de contrôle, par la valeur du motif voisin** : `git grep -n
"DETACHED_SUPPLIER_SETTLEMENT\|journal-entries-modify-blocked"` — le dernier motif ajouté par le
même geste ; chaque site rendu reçoit `ENTRY_LETTERED` ou est trié à la main.

⚠️ **La modification d'en-tête seul est refusée aussi** (révision de la clause (ii) d'AC7
d'août, C97) : `update_in_tx` réécrit toujours les lignes ; un chemin « en-tête seul » créerait
une seconde façon de modifier, pour un cas étroit (écriture **manuelle** lettrée). Délettrer,
modifier, relettrer reste possible tant que l'écriture se modifie (exercice ouvert, période non
verrouillée) — le délettrage lui-même suivant la règle des périodes de R7, qu'il satisfait toujours
quand `ENTRY_LETTERED` parle (ci-dessus).


**AC9 — Contre-passation (R6).** Tests nommés *(noms `snake_case` fixés en validation P4 — R4-8)*, par
la route `POST /journal-entries/{id}/reverse` :
(a) `reversal_letters_each_free_letterable_line_with_its_mirror` — écriture manuelle à deux lignes sur 1100 et 3200 (`Revenue`) non lettrée → après
contre-passation, la ligne 1100 et son miroir forment un groupe `reversal`, la ligne 3200 **n'est
pas** lettrée (compte non lettrable) — **assertions sur le corps du `201`** *(F-4, C116)* :
`letteringCode` non nul et égal à `code_from_key` sur la ligne miroir de 1100, nul sur la ligne
3200 ; puis la même chose en base ; (b) `reversal_leaves_an_existing_group_intact_and_the_mirror_open` — la même, dont la ligne 1100 est
lettrée `manual` avec une autre ligne → le groupe est **intact**, le miroir est **ouvert** ; (c) `reversal_of_an_entry_in_a_closed_year_letters_too` — origine sur
exercice **clos** → la contre-passation réussit et (a) vaut ; **et**, second cas du même test
*(validation P6, F6-1)*, origine en exercice ouvert mais datée **≤ `books_locked_through`** (borne
posée par SQL après la création de l'origine) → `201`, et l'origine **et** son miroir forment le
groupe `reversal` — le groupe n'est pas entièrement ≤ la borne, la vue « au X » ne bouge pas.
**Mutation tuée** : R6 qui évaluerait la règle des périodes sur l'origine (refus ou lettrage sauté
sous le verrou de période). Et
`reconciliation_cancel_letters_the_reversal_pair`, par le dé-rapprochement d'un rapprochement
**hors facture** (`cancel_in_tx`, `reconciliation_cancel.rs:278`, qui appelle `reverse_in_tx` à
`:370`) : la ligne 1100 et son miroir sont lettrés `reversal`. (d) `supplier_invoice_cancel_letters_a_pair_that_cannot_be_dissolved_by_hand` — l'annulation
d'une facture fournisseur validée et non payée (contre-passation de son écriture ; l'origine
garde le motif `OwnedBySupplierInvoice`) → groupe `reversal` posé au compte fournisseurs, puis
`DELETE /letterings/{key}` → 409 `LETTERING_LINE_OWNED_BY_DOCUMENT` (AC5 point 2, C106).
(e) `reversal_group_survives_retyping_and_bank_attachment_and_dissolves` — le groupe posé par (a) survit à un retypage du compte 1100 en `Expense` (`confirm_retype`)
et se dissout ensuite par `DELETE` → 204 (C104) ; idem après rattachement d'un `bank_accounts` au
compte. ⚠️ *Fixture* *(F-10)* : `confirm_retype` passe par `check_role_account_type`
(`kesh-db/src/repositories/accounts.rs:69`) — un compte portant un rôle (`Receivable`, …) n'accepte
que les types du rôle. Le test emploie un compte **sans rôle** (ou retire le rôle dans la fixture),
faute de quoi il échoue pour une cause étrangère au lettrage.
(f) `reverse_creates_the_opposite_entry_and_marks_the_origin_without_altering_it` — le test existant
`reverse_creates_the_opposite_entry_and_leaves_the_origin_intact` (`journal_entry_reversal_e2e.rs:263`),
**renommé et étendu** (R6, C129) : montants, comptes et `version` de l'origine inchangés **et** marque
posée sur sa ligne lettrable (clé égale à celle du miroir, origine `reversal`), aucune sur la ligne
`Expense` ; ses textes (docstring `:257-258`, commentaire `:281-283`, messages `:303`, `:318`) disent
désormais ce que dit son nom (R6 ; validation P5 — R5-6 = L-3). **Mutations tuées** : R6 qui ne lettre que le miroir, ou qui ne lettre rien (la marque de
l'origine manque) ; R6 qui réécrirait un montant ou un compte de l'origine (le tuple `account_id,
debit, credit` diffère).

**AC10 — Audit, part (ii).** Le lettrage `reversal` est audité par la primitive (`lettering.created`,
mêmes `details` que dans la 15-1a-i, dont `fiscalYearId`/`fiscalYearName` par ligne — C127 ; en mode
`System`, le nom vient de la lecture **ordinaire, non verrouillante** de R7 point 3 de la 15-1a-i —
validation P4, R4-1, C128), dans la transaction de la contre-passation ; l'acteur est l'auteur de la contre-passation, **sans** clé d'API
(`reverse_in_tx` ne la porte pas, `journal_entries.rs:2094` — écart nommé, R3 de la 15-1a-i). Test
nommé : `reversal_lettering_is_audited_by_the_reverser`.

**AC13 — Invariant, part (ii).** Le scénario de `lettering_invariants` (15-1a-i) gagne des groupes
`reversal` (contre-passation d'une écriture à lignes lettrables, dont une ligne déjà lettrée — R6,
seconde branche) ; l'invariant tient. Le commentaire de `COLONNES_DES_LIGNES`
(`journal_entries_modification.rs:78`) renvoie désormais à `lettering_guard` (AC8), et non plus à une
garde à venir.

**AC15 — Documentation, part (ii) : ce que les gardes changent à la modification, à la suppression et
à la contre-passation** *(validation P2 — R2-6 = F-7 ; rangs révisés en validation P3 — C126 ;
CHANGELOG — F3-2, C127 ; manuel — F3-3)*. **Sites d'`api-external.md` à toucher, nommés** *(aucun test
ne lit ce fichier : rien ne rougira s'il en manque un)* :

| site (au 2026-10-09, `5e4bec50`) | geste |
|---|---|
| `:223`, `:229`, `:255`, `:261` (prose) | réserve « lettrée » (ci-dessous) ; à `:229`, en plus, la parenthèse qui énumère ce que porte `modificationBlockedLabel` (« numéro de pièce, nom de l'exercice postérieur clos ou borne du verrou ») gagne « ou code de lettrage » (validation P4 — R4-3) |
| `:231` « 15 lignes » et son tableau (`:233-249`, en-tête compris ; `:232-247` jusqu'en P4 — R4-8) | une ligne `ENTRY_LETTERED` `409`, **en dernier**, après `PERIOD_LOCKED` (C126) ; intitulé **recompté** depuis le tableau |
| `:265` « 9 lignes » et son tableau (`:267-277` ; `:266-276` jusqu'en P4) | idem pour le `DELETE` : **en dernier**, après `PERIOD_LOCKED` (étape 3-quinquies) ; intitulé recompté |
| `:251` (« Les refus `409` d'une pièce portent `details.documentId` … et `details.documentNumber` (son numéro …) ») *(validation P5 — L-6)* | une phrase : « pour `ENTRY_LETTERED`, `details.documentId` est `null` et `details.documentNumber` porte le **code** du premier groupe de lettrage de l'écriture (ordre `id`), non un numéro de pièce » |
| `:279` (« Une écriture d'une pièce sous la période verrouillée répond par sa pièce … ») | une phrase : `ENTRY_LETTERED` parle **après** tous les autres refus — quand il parle, le délettrage (`DELETE /letterings/{key}`) aboutit |
| `:485` (catalogue des codes) | `ENTRY_LETTERED` |
| `:255`, **seul** site qui nomme `POST /journal-entries/{id}/reverse` (`grep -n "reverse" docs/api-external.md` → `:255`, `:359`, `:488` ; la route n'a ni section ni tableau propre) | *(tranché en validation P4 — F-7 : une phrase ici, pas de section neuve ; documenter la route en entier est hors du périmètre de cette story)* : « La contre-passation **lettre** ce qui est libre : chaque ligne de l'origine sur un compte lettrable, non encore lettrée, forme avec son miroir un groupe `reversal` ; la réponse `201` porte ces lignes lettrées (`letteringCode` non nul), et l'origine ne change que par cette marque — montants, comptes, dates et libellés intacts. » |

**CHANGELOG `[0.13.0]`** *(F3-2 de P3, C127 — aucun gate ne lit le CHANGELOG)* : les deux entrées #532
de *Modifié* (`CHANGELOG.md:19` et `:21` au 2026-10-09) énumèrent **déjà** le contrat du
`PUT`/`DELETE`, dans la même version non publiée. Elles sont **réécrites** : (1) dans l'énumération des
refus du `PUT` (« `200`, ou un refus nommé — `FISCAL_YEAR_CLOSED`, … , `PERIOD_LOCKED` »),
**`ENTRY_LETTERED`** (nouveau, `409`) **en dernier**, après `PERIOD_LOCKED` ; (2) « Ce qui reste figé,
et se corrige comme avant » gagne, à part, l'écriture **lettrée** : « une écriture dont une ligne est
lettrée se délettre d'abord (`DELETE /api/v1/letterings/{key}`), puis se modifie » ; (3) l'entrée de la
suppression (« aux mêmes conditions — exercice ouvert, … , date hors de la période verrouillée ») gagne
« aucune ligne lettrée » et hérite du refus. Et sous *Modifié*, une entrée « ⚠️ Changement de contrat
pour une intégration par clé d'API » : la **contre-passation lettre** ce qui est libre — la réponse `201`
de `POST /api/v1/journal-entries/{id}/reverse`, et l'effet des annulations qui contre-passent
(règlement, paiement fournisseur, facture fournisseur, rapprochement), portent des lignes **lettrées**
(`letteringCode` non nul) ; la contre-passation écrit une marque qu'elle n'écrivait pas — **y compris
sur les lignes de l'origine**, dont montants, comptes, dates et libellés restent intacts (C129). Les champs
neufs et les colonnes CSV sont annoncés par la 15-1a-i.

**Sites qui promettent « modifiable tant que l'exercice est ouvert » sans réserve** — relevés en
P1 par `grep` sur le `.tex` **et** sur le PDF aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '`),
**à re-greper au développement par la valeur** (`exercice (est|reste) ouvert`, `se modifie`,
`se supprime`) :

| support | sites (au 2026-10-09, `5e4bec50` — inchangés depuis le relevé de P1, sauf la FAQ) |
|---|---|
| `docs/manual/fr/user-manual.tex` | `:384`, `:481` (et la section `:483`), `:562`, `:628`, `:708`, `:744` (l'écriture d'ouverture, dont les lignes de bilan sont **lettrables**), `:758`, `:2224` (FAQ « revenir en arrière » ; `:2209` jusqu'en P3) |
| `docs/api-external.md` | `:223` (cadre de la modification), `:229` (`modificationBlockedBy`), `:255` (« une écriture refusée se corrige par contre-passation » — une écriture lettrée se **délettre** d'abord), `:261` (cadre de la suppression) ; tableaux `:231`/`:265` ci-dessus |
| `docs/manual/fr/admin-manual.tex` *(validation P4 — F-2 : absent du relevé de P1)* | `:1920` (clé `read-write` : « sous les mêmes gardes qu'à l'écran~: exercice ouvert, aucun exercice postérieur clôturé, aucune pièce propriétaire, période non verrouillée, … » — **énumération** : gagne « aucune ligne lettrée », en dernier ; la suppression « dans le même cadre » en hérite) ; `:1959` (« Une écriture se modifie tant que son exercice est ouvert … si aucun exercice postérieur n'est clôturé, qu'aucune pièce ne la possède et hors période verrouillée » — **énumération** : idem, et la phrase sur la suppression) ; `:2093` (art. 957a CO al. 1 : « se modifie tant que son exercice est ouvert … elle devient définitive à la clôture ») — réserve « sauf si l'une de ses lignes est lettrée, auquel cas elle se délettre d'abord » ; `:2128` (OLICo art. 3 : « ou --- tant que l'exercice est ouvert --- par \textbf{modification tracée} » — validation P5, R5-2 = L-5) — même réserve |
| `README.md` *(F-2)* | `:29` (« écritures validées, modifiables et supprimables tant que l'exercice est ouvert ») — réserve brève : « … tant que l'exercice est ouvert et qu'elles ne sont pas lettrées » |
| **catalogues i18n et replis** *(validation P5 — R5-1 : deux textes d'**écran**, jumeaux de `user-manual.tex:744`/`:758`, que le relevé de P4 ne visait pas)* | `opening-balances-complete-confirm` — `fr-CH:985`, `de-CH:934`, `en-CH:934`, `it-CH:934`, repli `frontend/src/routes/(app)/settings/opening-balances/+page.svelte:233-234` (« Elle reste modifiable depuis sa fiche tant que l'exercice est ouvert ») ; `opening-balances-locked-already-has-entries` — `fr-CH:952`, `de-CH:901`, `en-CH:901`, `it-CH:901`, repli `:386-388` (« en modifiant l'écriture d'ouverture tant que l'exercice est ouvert ») — textes arrêtés ci-dessous. L'écriture d'ouverture et le complément portent des lignes de bilan, **lettrables** : lettrer le solde d'ouverture d'un compte débiteurs avec les encaissements qui suivent est le cas d'usage type |

| clé | locale | texte réécrit (le segment changé est la réserve finale ; apostrophe typographique `’`, comme le catalogue) |
|---|---|---|
| `opening-balances-complete-confirm` | fr-CH | Enregistrer cette écriture de complément, datée du { $date } ? Elle reste modifiable depuis sa fiche tant que l’exercice est ouvert — sauf si l’une de ses lignes est lettrée : délettrez-la d’abord. |
| | de-CH | Diese Ergänzungsbuchung mit Datum { $date } speichern? Sie bleibt über ihre Detailansicht änderbar, solange das Geschäftsjahr offen ist — ausser wenn eine ihrer Zeilen ausgeglichen ist: Heben Sie den Ausgleich zuerst auf. |
| | en-CH | Save this complement entry, dated { $date }? It remains editable from its detail page while the fiscal year is open — unless one of its lines is matched: unmatch it first. |
| | it-CH | Salvare questa registrazione di completamento, datata { $date }? Resta modificabile dalla sua scheda finché l’esercizio è aperto — salvo se una delle sue righe è abbinata: annulla prima l’abbinamento. |
| `opening-balances-locked-already-has-entries` (segment « en modifiant … ouvert » seul ; le reste inchangé) | fr-CH | … se corrige dans le journal, en modifiant l’écriture d’ouverture tant que l’exercice est ouvert (après l’avoir délettrée si l’une de ses lignes est lettrée), ou par une contre-passation ou une écriture de correction. |
| | de-CH | … wird im Journal korrigiert, durch Ändern der Eröffnungsbuchung, solange das Geschäftsjahr offen ist (nach Aufhebung des Ausgleichs, falls eine ihrer Zeilen ausgeglichen ist), oder durch eine Stornobuchung oder eine Korrekturbuchung. |
| | en-CH | … is corrected in the journal, by modifying the opening entry while the fiscal year is open (after unmatching it if one of its lines is matched), or by a reversal or a correcting entry. |
| | it-CH | … si corregge nel giornale, modificando la registrazione di apertura finché l’esercizio è aperto (dopo averne annullato l’abbinamento se una delle sue righe è abbinata), oppure con uno storno o una registrazione di rettifica. |

⚠️ *Terminologie* : comme pour le dialogue de R6, « ausgleichen / Ausgleich », « match / unmatch »,
« abbinare / abbinamento » s'alignent sur les traductions que la 15-1a-i aura retenues — une seule
désignation du lettrage par langue. **Trié, non touché** : `error-opening-complement-account-moved`
(`fr-CH:1002`, `de/en/it-CH:951`, repli Rust `kesh-api/src/errors.rs:1253`) énumère trois voies
(« en modifiant l'écriture, par une contre-passation ou une écriture de correction ») sans promettre la
première sous condition d'exercice — il ne la promettait déjà pas sous le verrou de période ; les deux
autres voies restent ouvertes à une écriture lettrée. `user-manual.tex:1351` (dévalidation) : trié plus
bas. `docs/kesh-specifications.txt` (FR23, FR86, « tant que l'exercice est ouvert ») : exigences du
PRD, non une promesse d'écran ni de manuel.

Chacun reçoit la réserve « sauf si l'une de ses lignes est **lettrée** : délettrez-la d'abord ».
⚠️ **Sauf la liste des conditions du bouton** *(F3-3 de P3)* : `user-manual.tex:490-508` (« Le bouton
n'est offert que si **toutes** ces conditions tiennent » suivi d'un `itemize` de cinq points) est une
**énumération qui se dit exhaustive** — une réserve en prose à côté la laisserait fausse. Elle gagne un
**sixième** `\item`, **en dernier** (le rang d'AC8) : « \textbf{aucune de ses lignes n'est lettrée} ---
sinon, délettrez-la d'abord (le délettrage se fait par l'API dans cette version), puis modifiez ; quand
c'est le seul motif, le délettrage aboutit, sauf geste concurrent sur les mêmes lignes » ; le
**cinquième** point (`:506`, « ni contre-passée, ni elle-même une contre-passation} ») passe de `.` à
`~;`, le sixième prenant le point final *(validation P6, F6-2)* *(« aboutit
toujours » jusqu'en P4 : absolu là où AC8 écrit « hors concurrence » — R4-4)* ; **et la phrase qui suit
la liste**, `:509-513` (« Sinon, la fiche affiche le motif à la place du bouton, et la correction passe
par une contre-passation (ci-dessous) --- ou, pour une pièce, par le chemin de la pièce »), gagne le cas
lettré : « --- ou, pour une écriture lettrée, par le délettrage » *(validation P4 — R4-4 : pour ce
motif, la voie n'est pas la contre-passation)* ; `:531-534` (« Quand l'écriture ne se modifie pas, elle ne se
supprime pas non plus … et affiche le motif ») couvre la suppression sans retouche. `:1349` (dévalidation
d'une facture) n'est **pas** concerné (une facture lettrée a un règlement ou un avoir, déjà refusés).

Toucher le `.tex` impose de **régénérer et versionner** le PDF (`make fr` dans `docs/manual/`), puis
de le contrôler **aplati** — **les deux** PDF, `user-manual.pdf` **et** `admin-manual.pdf` *(F-2 : le
contrôle ne portait que sur le manuel utilisateur)* : zéro occurrence restante de la promesse sans
réserve (re-grep par la valeur : `se modifie`, `mêmes gardes`, `modifiables et supprimables`,
`exercice (est|reste) ouvert`), zéro occurrence de `pas touchée` dans la section de la contre-passation
*(F5-1 ; sans l'apostrophe, typographique dans le PDF)*, « toutes ces conditions tiennent » suivi de
**six** points. Le `README.md` se contrôle par le même `grep`. ⛔ **Et les catalogues i18n et les
replis** *(validation P5 — R5-1 : le contrôle s'arrêtait aux manuels et au README, et rien ne ramenait
les textes d'écran)* : `git grep -nE "tant que l.exercice|reste modifiable|en modifiant|solange das
Geschäftsjahr|while the fiscal year is open|finché l.esercizio|n.en modifie aucune|ne modifie rien|reste
intacte|untouched|unverändert|resta intatta" -- crates/kesh-i18n frontend/src crates/kesh-api/src` —
chaque occurrence porte la réserve « lettrée » ou est triée ci-dessus.

## Tasks

- [x] **T0 (part ii)** (AC8) — Relevés au sol sur le `main` du moment, la 15-1a-i mergée : `EXPLAIN`
      de la requête de `lettering_guard` (attendu : `ref` sur `idx_jel_entry` ou `uq_jel_entry_order`) ;
      constater si la 15-12b est mergée (précédence de `delete_in_tx`, AC8, C117 :
      `grep -n "enforce_ownership" crates/kesh-db/src/repositories/journal_entries.rs` autour de
      l'étape 2-bis) ; relire au code l'ordre réel des étapes de `delete_in_tx`, `update_in_tx` et
      `modification_blocker` (les numéros d'AC8 datent de `5e4bec50`). Résultats au Dev Agent Record.
- [x] **T4** (AC8) — `ModificationGuard::Lettered { code }` (`code`, `document_id`, `label`) ;
      fonction `lettering_guard(conn, company_id, id, Lecture)` ; appels **inconditionnels** dans
      `delete_in_tx` (étape **3-quinquies**, après le verrou de période, hors `enforce_ownership`) et
      `update_in_tx` (étape **7-bis**, après le verrou de période, avant l'étape 8), `Lecture::Conseil`
      dans `modification_blocker` (**après** `PeriodLocked`) ; requête sans `ORDER BY … LIMIT` ; bras du
      mapping 409 `ENTRY_LETTERED` (`kesh-api/src/errors.rs:2822`) ; « onze » → « douze »
      (`kesh-db/src/errors.rs:206`, `kesh-api/src/routes/journal_entries.rs:163-166`) ; doc-comments
      « Ordre des refus » de `delete_in_tx` (`:1593-1610`, étape 3-quater `:1723-1726`), d'`update`
      (`:1248-1258`), de `modification_blocker` (`:1068-1071`) et d'`invoices::unvalidate`
      (`invoices.rs:1487` — chaîne finissant par `PERIOD_LOCKED → ENTRY_LETTERED`, L6) ; **et les doc-comments
      de P4** (R4-2, F-7, R4-3) : `kesh-db/src/errors.rs:121-122`, `:189-191`, `:199`, `:241-244`,
      `journal_entries.rs:1640-1641`, `invoices.rs:1471-1473`, `:1650-1657`,
      `kesh-api/src/routes/journal_entries.rs:168-169` ; **et ceux de P5** (R5-3 = L-2, R5-5 = L-1, R5-8,
      C132) : `kesh-db/src/errors.rs:152-153`, `:161`, `:172`, `:216-217` ; cadre « aucune ligne lettrée »
      à `routes/journal_entries.rs:711-714`, `journal_entries.rs:25-27`, `:44-48`, `:1161-1164`,
      `:1552-1555` ; « six lectures » `:1073` ; `invoices.rs:1471-1473` et `kesh-db/src/errors.rs:241-244`
      gardent « trois » et **nomment** la marque, inatteignable (C132) ; **les textes de l'ordre des
      verrous et du cycle** (F5-2 = R5-4, tableau d'AC8) : `docs/MULTI-TENANT-SCOPING-PATTERNS.md:326`,
      `:327`, `:331`, `:332`, `routes/journal_entries.rs:723-728`, `journal_entries.rs:1205-1209`,
      `:1218-1234`, `:1619-1630`, `crates/kesh-api/tests/audit_route_registry.rs:80-83` et `:200-202`
      (R6-1), et un commentaire au verrou des lignes de l'origine ajouté par R6
      (`:2266`) ; relevé par la valeur du tableau d'AC8 rejoué ; test `each_screen_code_maps_to_its_put_and_delete_refusal`
      étendu à douze codes (F-1) ; si la 15-12b
      est mergée : précédence 2-bis → 3-quinquies écrite et testée par la paire d'AC8 (C117).
- [x] **T4-bis** (AC8, écran) — `frontend/src/lib/features/journal-entries` :
      `ModificationBlocker` (`journal-entries.types.ts:91`, `| 'ENTRY_LETTERED'` **en dernier**, après
      `'PERIOD_LOCKED'` `:96` — le doc `:85` dit « dans l'ordre de précédence du serveur », et la marque
      parle en dernier, F6-3 ; doc `:85` « onze » → « douze » ; doc de `modificationBlockedLabel`
      `:77-80`, « ou code de lettrage » — R4-3), `modificationBlockerLabel`
      (`blocker-messages.ts:81` ; sa doc `:76-79`, « ou le code de lettrage » — R5-3 = L-2),
      `deleteJournalEntry` (doc `journal-entries.api.ts:69-71`, « aucune ligne lettrée » — R5-5),
      `editRefusalOutcome` (`form-helpers.ts:108-130` → `'stale'`), et
      leurs tests (`blocker-messages.test.ts` `ATTENDU` 11 → 12, `ENTRY_LETTERED` en dernière entrée,
      après `PERIOD_LOCKED` — F6-3 ; `:24`, `:50`, `:52`, `:53` — R4-7,
      `form-helpers.test.ts`,
      `JournalEntryForm.edit.test.ts:132`), commentaire `i18n-libelle-en-dur.test.ts:160-165`
      (douze codes, **cinq** branches propres).
- [x] **T5** (AC9, R6) — Lettrage `reversal` en fin de `reverse_in_tx_inner` : lecture des lignes
      d'origine étendue (`id`, `lettering_key`, `FOR UPDATE`, R6), appariement par **position**,
      test « déjà lettrée » et lettrabilité (`letterings::is_letterable_account` — F-9) **avant** l'appel,
      puis `create_group_in_tx`
      (origine `Reversal`, mode `System { exercice du jour }`, sans le refus R5) ; **relecture des
      lignes de l'écriture inverse** si un groupe a été posé, avant le retour (R6, C116). **Et ce que
      « intacte » veut dire** (R6, C129) : doc-comment `routes/journal_entries.rs:477-479`, clé
      `journal-entries-reverse-dialog-body` (quatre locales, texte arrêté à R6), repli
      `+page.svelte:475`, doc de `reverseJournalEntry` (`journal-entries.api.ts:42-45`, texte arrêté
      à R6 — R6-2), **et `user-manual.tex:608`** (texte arrêté à R6 : l'origine « n'est pas
      modifiée » dans ses montants, comptes, date et libellé, et la contre-passation **lettre** —
      validation P5, F5-1).
- [x] **T10 (part ii)** (AC8) — i18n, **une** clé dans les **quatre** locales, avec son repli FR
      identique au catalogue :

      | code | clé | texte FR |
      |---|---|---|
      | `ENTRY_LETTERED` | `journal-entries-modify-blocked-lettered` | Cette écriture est lettrée : délettrez-la d’abord. |

      (apostrophe typographique `’` au catalogue et au repli frontend, comme
      `journal-entries-modify-blocked-*` `fr-CH:375-378` et `blocker-messages.ts` ; le repli Rust du
      mapping suit son voisin, `'` droit à `kesh-api/src/errors.rs:2832` — validation P5, L-8)

      Les gardes `i18n-keys.test.ts` / `i18n-un-repli-par-cle.test.ts` voient leurs comptes figés
      **recomptés** depuis leur source (la 15-1a-i y a déjà porté ses dix clés et le lot d'audit).
      ⚠️ **Et une clé existante réécrite** (R6, C129) : `journal-entries-reverse-dialog-body`, quatre
      locales et repli `+page.svelte:475`, texte arrêté à R6 — aucun compte figé n'en bouge.
      ⚠️ **Et deux de plus** (validation P5 — R5-1) : `opening-balances-complete-confirm` et
      `opening-balances-locked-already-has-entries`, quatre locales et replis
      `settings/opening-balances/+page.svelte:233-234`, `:386-388`, textes arrêtés à AC15 (ii) — aucun
      compte figé n'en bouge.
- [x] **T11 (part ii)** (AC15 part ii) — `api-external.md` (sites du tableau d'AC15 part ii : `:223`,
      `:229` et sa parenthèse du label, `:231`/`:265` et leurs intitulés recomptés, `:255` et la phrase
      de la contre-passation, `:261`, `:279`, `:485`), CHANGELOG (entrées #532 réécrites ; *Modifié* :
      contre-passation qui lettre, origine marquée), sites du manuel utilisateur d'AC15 part ii — dont le
      **sixième** `\item` de `:490-508` (le cinquième, `:506`, passant de `.` à `~;` — F6-2) et la
      phrase `:509-513` (R4-4) —, **manuel d'administration**
      (`:1920`, `:1959`, `:2093`, `:2128` — R5-2 = L-5) et **`README.md:29`** (F-2), la phrase de
      `api-external.md:251` (L-6), `user-manual.tex:608` (F5-1, avec T5), et les **deux** PDF régénérés
      puis contrôlés aplatis (`user-manual.pdf`, `admin-manual.pdf`) ; enfin le **contrôle final étendu
      aux catalogues i18n et aux replis** (R5-1, commande à AC15 ii).
- [x] **T12 (part ii)** — Tests :
      - **précédence, par paire, sur chaque chemin** *(C126 — mutation tuée sur chacun : « la marque
        avant le verrou de période »)* ; montage : une écriture **manuelle** lettrée avec une ligne
        d'une autre écriture **en période ouverte** (groupe légal), puis la borne
        `books_locked_through` posée **après** le lettrage :
        - `delete_of_a_lettered_entry_in_a_locked_period_says_period_locked` : écriture datée ≤ la
          borne → `DELETE` (route) **et** `delete_in_tx(…, false)` → `PERIOD_LOCKED` ; datée après →
          `ENTRY_LETTERED` ;
        - `update_of_a_lettered_entry_in_a_locked_period_says_period_locked` : `PUT` → `PERIOD_LOCKED`
          (ancienne date ≤ borne) ; et une nouvelle date ≤ borne sur une écriture datée après →
          `PERIOD_LOCKED` ; sans borne → `ENTRY_LETTERED` ;
        - `blocker_of_a_lettered_entry_in_a_locked_period_is_period_locked` : `GET` →
          `modificationBlockedBy = "PERIOD_LOCKED"` ; datée après → `"ENTRY_LETTERED"` ;
        - `update_of_a_lettered_entry_with_a_stale_version_says_conflict` : `PUT` à version périmée →
          `OPTIMISTIC_LOCK_CONFLICT` (le rang de P2 rendait `ENTRY_LETTERED`) ;
      - `entry_lettered_refusal_leads_to_a_dissolution_that_succeeds` : `PUT` → `ENTRY_LETTERED` ;
        `DELETE /letterings/{key}` → `204` (la route reçoit le code, R2 de la 15-1a-i — écriture unifiée
        en validation P5, R5-9 = L-9) ; `PUT` → `200` — la promesse du message, tenue ;
      - AC8 : `PUT`, `DELETE`, en-tête seul, `PUT` identique (no-op) → `ENTRY_LETTERED` ;
        **`delete_in_tx(…, enforce_ownership = false)` sur une écriture lettrée posée directement en
        base → refus `Lettered`** ; `GET` de la fiche → `modificationBlockedBy = "ENTRY_LETTERED"` et
        `modificationBlockedLabel` = le code ; la **paire de précédence** avec la 15-12b si elle est
        mergée (C117) ;
      - R7 (côté gardes) : `POST /letterings` concurrent d'un `PUT` sur l'une des écritures → l'un des
        deux réussit, l'autre rend un refus métier (`ENTRY_LETTERED` ou ligne introuvable → 404),
        **jamais** un 500 — motifs de `attendre_une_requete_en_cours` : ceux de R7 point 2 de la 15-1a-i
        (`["jel.id IN", "FOR UPDATE"]` pour l'acte 1), jamais `["je.fiscal_year_id", "FOR UPDATE"]`,
        que l'étape 2 de `delete_in_tx` et l'acte 1 du lettrage contiennent tous deux (F3-6) ;
      - AC9 : les six tests nommés — (a) à (e) et `reconciliation_cancel_letters_the_reversal_pair`
        (R6-5) —, (c) avec son second cas sous période verrouillée (F6-1), dont l'assertion sur le **corps** du `201` (a), et (f),
        le test existant renommé `reverse_creates_the_opposite_entry_and_marks_the_origin_without_altering_it`
        (C129), **textes compris** (`:257-258`, `:281-283`, `:303`, `:318` — R5-6 = L-3) ;
      - AC8 / F-1 : `each_screen_code_maps_to_its_put_and_delete_refusal` à **douze** codes ;
      - AC10 : `reversal_lettering_is_audited_by_the_reverser` ;
      - AC13 : `lettering_invariants` avec des groupes `reversal`.
      ⚠️ *Tests existants qui changent de forme* (axe signalé non exercé par la lentille F de P3 ;
      liste corrigée en validation P4 — R4-5, relevé `grep -nE "async fn .*(deadlock|waits_for|attend|concurrent)"`
      sur `5e4bec50`) : R6 ajoute deux acquisitions de verrou dans `reverse_in_tx_inner` (lignes de
      l'origine, acte 1 de la primitive), et T4 une lecture verrouillante des lignes dans
      `update_in_tx` / `delete_in_tx`. Relus un par un, et passés au gate complet :
      (1) les tests où une modification ou une suppression **croise** une contre-passation —
      `update_waits_for_a_concurrent_reversal_then_refuses` (`journal_entries_modification.rs:346`),
      `delete_waits_for_a_concurrent_reversal_then_refuses` (`:451`),
      `update_and_a_reversal_of_the_same_year_can_deadlock` (`:545`) ; (2) celui qui **rejoue une
      victime** d'interblocage au travers d'une annulation qui contre-passe —
      `settlement_cancellation_is_replayed_when_it_is_the_deadlock_victim`
      (`rejeu_interblocage_e2e.rs:626`) ; (3) ceux où une **clôture attend** une annulation qui
      contre-passe, sans interblocage — motif `["fiscal_years", "FOR UPDATE"]`, que les verrous ajoutés
      ne doivent pas rendre muet : `a_concurrent_close_waits_for_the_unreconciliation`
      (`reconciliation_cancel.rs:128`), `une_cloture_concurrente_attend_l_annulation`
      (`invoice_settlement.rs:1207`), `a_concurrent_close_waits_for_the_cancellation` et
      `a_concurrent_close_waits_for_the_invoice_cancellation` (`supplier_invoices_repository.rs:1657`,
      `:1963`) ; (4) les tests en module de `crates/kesh-db/src/repositories/journal_entries.rs` qui
      appellent `reverse_in_tx` sur base partagée *(validation P6, R6-4)* — l'auxiliaire
      `supprimer_avec` (`:3202`, appel `:3219`) et les tests qui l'emploient, et
      `reverse_in_tx_disparait_avec_le_rollback_de_l_appelant` (`:4856`, appel `:4891`), dont le
      rollback doit emporter aussi le groupe `reversal` et l'audit `lettering.created`. Non exécuté à la spécification : leur tenue est une hypothèse, que seul le gate tranche.

## Dev Notes

- **Gate `kesh-db` : complet, jamais ciblé** (repository `journal_entries`, P6/P7). Base remise à zéro
  **avant** le gate (KF-039) — par `DROP/CREATE DATABASE` de **ses** bases, jamais par redémarrage du
  conteneur. E2E complet au dernier commit de code (D7) : l'écran de la fiche d'écriture est touché
  (AC8). Pas de migration ici : le bump `min_required` et le bump Cargo sont dans la 15-1a-i.
- **Dépendances** : ⛔ **la 15-1a-i, mergée** (section « Ce que cette fiche reçoit »), et avec elle la
  15-12a. La **15-12b** passe avant de préférence : elle touche `delete_in_tx` (étape 2-bis
  inconditionnelle) — précédence C117, écrite et testée par la seconde des deux à merger. Également :
  15-8a et 15-8b (mergées — `modification_guard`, `delete_in_tx`, `update_in_tx`), 15-5e1/e2 (mergées
  — enveloppes de `reverse` et des annulations). **En aval** : la 15-1a2 (lettrage des pièces) suppose
  R6 — les annulations de pièce dissolvent le groupe `document` puis contre-passent, et la
  contre-passation lettre ce qui est libre.
- **Règle de découpage** : cette fiche **est** le produit d'un découpage (C124, couture de C118). Modules
  touchés, au grain « crates Rust, packages npm » : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` =
  **4**. Au grain des modules métier — **recompté en validation P4** (F-6 : « sept, dont trois de
  commentaire » était faux dans les deux sens) : `repositories/journal_entries` (garde,
  contre-passation), `kesh-db/errors` (variante `Lettered`, `code()`/`document_id()`/`label()`, et
  doc-comments), `kesh-api/errors` (bras du mapping), `routes/journal_entries` (doc-comments),
  `kesh-i18n` (une clé neuve, une réécrite), `features/journal-entries` (types, libellés, refus),
  `routes/(app)/journal-entries` (repli du dialogue), `repositories/invoices` (doc-comments),
  `kesh-api/tests` (table de correspondance, test de contre-passation), `docs` (deux manuels, API,
  Pattern 5), et, **depuis la validation P5** (R5-1), `routes/(app)/settings/opening-balances` (deux
  replis de texte) — **onze**, dont **deux** de commentaire seul (`repositories/invoices`,
  `routes/journal_entries`) et **un** de texte seul (`settings/opening-balances`). **Le seuil « plus de 5 » est franchi** ; signal **déclaré au Project Lead**. Le
  découpage C124 en est déjà la réponse ; un second séparerait la garde (AC8) de la contre-passation qui
  lettre (R6), qui partagent `repositories/journal_entries`, le test d'invariant et la règle de
  publication — non retenu, à l'arbitrage de l'orchestrateur.
- **FR86** : la lecture de C94/C105/C113 (15-1a-i, Dev Notes) vaut ici — le gel dit « délettrez
  d'abord », et le délettrage suit la règle des périodes.
- Références : `journal_entries.rs` (`:1009`, `:1058`, `:1068-1071`, `:1077`, `:1130`, `:1248-1258`,
  `:1307-1309`, `:1391`, `:1395`, `:1419-1432`, `:1433`, `:1459`, `:1480`, `:1593-1610`, `:1719`,
  `:1723-1745`, `:1922`, `:2094`, `:2175`, `:2266`, `:2349`), `kesh-db/src/errors.rs:130`, `:206`,
  `kesh-api/src/errors.rs:1145`, `:2822`, `kesh-api/src/routes/journal_entries.rs:163-166`, `:461`,
  `:490-497`, `invoices.rs:1487`, `:1530-1607`, `:1660`, `reconciliation_cancel.rs:278/370`,
  `accounts.rs:69`, `audit_route_registry.rs:199/203/205` ; ajoutés en P4 : `kesh-db/src/errors.rs:121-122/189-191/199/241-244`,
  `journal_entries.rs:1640-1641/2187`, `invoices.rs:1471-1473/1650-1657`,
  `kesh-api/src/routes/journal_entries.rs:168-169/477-479/690`, `journal_entry_reversal_e2e.rs:263/306-319/2102/2108/2218-2219`,
  `+page.svelte:475`, `admin-manual.tex:1920/1959/2093`, `README.md:29`, `user-manual.tex:509-513` ;
  ajoutés en P5 : `kesh-db/src/errors.rs:152-153/161/172/216-217`, `journal_entries.rs:25-27/44-48/1073/1161-1164/1205-1209/1218-1234/1552-1555/1619-1630`,
  `kesh-api/src/routes/journal_entries.rs:711-714/723-728/736`, `kesh-api/src/errors.rs:1253/2832`,
  `blocker-messages.ts:76-79`, `journal-entries.api.ts:69-71`, `settings/opening-balances/+page.svelte:233-234/386-388`,
  `messages.ftl` (`fr-CH:952/985`, `de/en/it-CH:901/934`), `journal_entry_reversal_e2e.rs:257-258/281-283/303/318`,
  `docs/MULTI-TENANT-SCOPING-PATTERNS.md:326/327/331/332`, `api-external.md:251/288-299`,
  `user-manual.tex:608`, `admin-manual.tex:1919/1961/2128`, `README.md:220`, `invoices.rs:1376`,
  `invoices/[id]/+page.svelte:355` ; ajoutés en P6 : `audit_route_registry.rs:80-83/200-202` (module
  `kesh-api/tests`, déjà compté), `journal-entries.api.ts:42-45`, `journal-entries.types.ts:96`,
  `user-manual.tex:506`, `journal_entries.rs:3202/3219/4856/4891` (tests en module) — aucun module neuf,
  toujours **onze**. ⚠️ **Tous les numéros se vérifient sur
  `origin/main`** (`5e4bec50` au 2026-10-09 ; `ec745d0c` depuis, sans changement dans les fichiers
  cités — `git diff --stat 5e4bec50 ec745d0c` ne les touche pas), non sur la branche de planification.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (Claude Code, sous-agent de développement de l'Epic 15), le 2026-10-09 — worktree
`/home/gcorbaz/devel/kesh-15-1a-ii`, branche `story/15-1a-ii-gardes-du-lettrage`, partie de
`origin/main` `0724904c` (15-1a-i mergée).

### Debug Log References

- T0, `EXPLAIN SELECT id, lettering_key FROM journal_entry_lines WHERE entry_id = 1 FOR UPDATE`
  sur `kesh_151aii` : `type = ref`, `key = uq_jel_entry_order` (possibles :
  `uq_jel_entry_order, idx_jel_entry`) — l'attendu de la fiche.
- Gates et mutations : `/home/gcorbaz/devel/kesh-gate-logs/15-1a-ii-gate-dev.log`,
  `15-1a-ii-e2e.log`, `15-1a-ii-mutations.md`, `15-1a-ii-make-fr.log`.

### T0 — la fiche relue contre le code de `0724904c` : écarts

1. **Numéros de ligne** : tous décalés (15-12a, 15-12b, 15-1a-i mergées depuis `5e4bec50`). Au
   code de `0724904c` : `modification_blocker` `journal_entries.rs:1126` ; `update_in_tx` `:1343`,
   étape 7 `:1468-1480`, étape 8 `:1482` ; doc « Ordre des verrous » d'`update` `:1254-1289`,
   « Ordre des refus » `:1297-1307` ; `delete_in_tx` `:1697`, étape 3-quater `:1771-1792`, doc
   « Ordre des refus » `:1642-1667`, « Sérialisation » `:1669-1681`, « qui passe quoi »
   `:1683-1692` ; `reverse_in_tx_inner` `:2237`, lecture des lignes de l'origine `:2327` ;
   mapping 409 `kesh-api/src/errors.rs:2912` ; `kesh-db/src/errors.rs` : `ModificationGuard`
   `:116-180`, `UnvalidationBlocker` `:228-246`. Chaque site a été retrouvé par la valeur.
2. **La 15-12b est mergée** : l'étape 2-bis de `delete_in_tx` est inconditionnelle → la paire de
   précédence C117 revient à **cette** story (T4) — faite :
   `delete_of_a_lettered_entry_under_a_later_closed_year_says_later_closed`.
3. **Totaux de la dévalidation** (réserve R6-3) : sur `0724904c`, la 15-12b a écrit « quatre
   autres » à la doc d'`unvalidate` (`invoices.rs:1471-1473`) et « neuf » aux totaux
   (`invoices.rs:1376`, `invoices/[id]/+page.svelte:355`, `admin-manual.tex`). Ces totaux
   restent tels quels (C132) ; la doc d'`unvalidate` garde « quatre » et **nomme** la marque. ⚠️
   **Écart** : `kesh-db/src/errors.rs` (doc d'`UnvalidationBlocker`) disait encore « trois autres
   empêchements », la 15-12b ne l'ayant pas mis à jour ; corrigé en « quatre » (exercice
   postérieur clos ajouté) avec la marque nommée et non comptée — C-15-1a-ii-1.
4. **Les six textes provisoires de la 15-1a-i** (section « Reçu de la 15-1a-i ») retrouvés par la
   forme resserrée : `CHANGELOG.md:15`, `docs/api-external.md:223`, `:289`, `:322`, `:323`,
   `user-manual.tex:2424-2425` (et le PDF). Ceux du `reversal` sont réécrits au commit du
   comportement (R6) — la contradiction avec AC15 (ii) est levée ; ceux du `document`
   (`api-external.md` : « `document` est réservé » à `:223` et `:289`, l'annotation de
   `LETTERING_IS_DOCUMENT` à `:322` ; le « une facture soldée par ses règlements » du CHANGELOG et
   du glossaire) restent à la 15-1a2.
5. **Qui ferme #518** : la tâche annonçait que cette story « clôt » #518 ; la fiche, elle, dit
   `refs #518`, et la 15-1c se déclare « closes #518 (dernière des quatre) » — l'issue est la
   fonctionnalité entière (« savoir ce qui reste ouvert sur un compte »), dont l'écran viendra à
   la 15-1c. **La PR de cette story porte `refs #518`, non `closes`** ; le commentaire « la
   15-1a-ii fermera l'issue » de la clé 15-1a-i du sprint-status est rectifié — C-15-1a-ii-3.

### Completion Notes List

- **AC8 — le gel.** `ModificationGuard::Lettered { code }` (`code()` = `ENTRY_LETTERED`,
  `document_id()` = `None`, `label()` = le code) ; `journal_entries::lettering_guard(conn,
  company_id, id, Lecture)` — une requête (`SELECT id, lettering_key … WHERE entry_id = ?`, sans
  `ORDER BY`, `LIMIT` ni filtre ; `FOR UPDATE` en `Lecture::Verrouillante`), premier groupe
  choisi en Rust par `id`. Appels : `delete_in_tx` étape **3-quinquies** (après 3-quater, hors
  `enforce_ownership`), `update_in_tx` étape **7-bis** (après le verrou de période, avant
  l'instantané et le no-op), `modification_blocker` en dernier (`Lecture::Conseil`). Bras
  `ENTRY_LETTERED` du mapping 409 (forme commune, `documentId` nul, `documentNumber` = code).
  Doc-comments réécrits partout où la fiche les nomme (douze codes, six lectures, 3-quinquies,
  ordre des verrous et cycle lignes ↔ écriture, chaîne `… → PERIOD_LOCKED → ENTRY_LETTERED`).
- **AC8 — l'écran.** Union `ModificationBlocker` + `'ENTRY_LETTERED'` en dernier ; branche de
  `modificationBlockerLabel` (repli FR = catalogue) ; `editRefusalOutcome` → `'stale'` ;
  commentaires des onze → douze codes ; compte figé `sitesTotal` 1919 → 1920 (`blocker-messages.ts`
  12 → 13 `i18nMsg(`, recompté aux deux bornes).
- **AC9 — R6.** Lecture des lignes de l'origine étendue (`id`, `lettering_key`, `FOR UPDATE`) ;
  boucle par **position** (zip des lignes de l'origine et de la contre-passation, toutes deux par
  `line_order`) ; ligne déjà lettrée → `continue` ; `letterings::is_letterable_account` avant
  l'appel ; `create_group_in_tx(… Origin::Reversal, Mode::System { held_open_fiscal_year_id:
  fy.id }, Actor { api_key_id: None })` ; **relecture** des lignes de l'écriture inverse si un
  groupe a été posé (C116). Les textes « n'en modifie aucune » réécrits aux cinq endroits (doc de
  la route, clé du dialogue en quatre locales et son repli, doc de `reverseJournalEntry`, manuel
  `:615`).
- **AC10, AC13.** Audit `lettering.created` par la primitive, acteur = auteur, sans clé (test
  `reversal_lettering_is_audited_by_the_reverser`) ; `lettering_invariants` gagne trois groupes
  `reversal` et un manuel (dont une écriture à ligne déjà lettrée : groupe intact, miroir
  ouvert), 5 → 9 groupes, invariant tenu ; commentaire de `COLONNES_DES_LIGNES` renvoie à
  `lettering_guard`.
- **AC15 (ii).** `api-external.md` (cadre, label, tableaux `PUT` 15 → **16** lignes et `DELETE`
  9 → **10** — recomptés depuis les tableaux —, `details` d'`ENTRY_LETTERED`, contre-passation
  qui lettre, origines, annotation `reversal` retirée, catalogue des codes) ; CHANGELOG (entrée du
  lettrage : la contre-passation lettre, l'écriture lettrée est figée ; entrées #532 réécrites ;
  *Modifié* : la contre-passation lettre, origine marquée) ; manuel utilisateur (sixième `\item`,
  cinquième passé à `~;`, phrase du chemin de correction, sites (numéros de `0724904c`) `:389`, `:486`, `:566`, `:615`,
  `:635`, `:719`, `:770`, `:784`, FAQ, glossaire, et — trouvé au contrôle du PDF aplati, hors
  relevé de la fiche — la phrase de la réouverture `:740`, C-15-1a-ii-6) ; manuel
  d'administration (`:2061`, `:2100` deux phrases, `:2238`, `:2273`) ; `README.md:29` ; Pattern 5
  (lignes `PUT`/`DELETE` et leurs notes de cycle) ; deux clés d'écran des soldes de départ (quatre
  locales, deux replis). PDF utilisateur et administration régénérés (`make fr`), brochure
  rendue à sa version (octets d'horodatage seuls).
- **Contrôles par la valeur** (après le dernier patch) : `pas touchée` ×0 dans la section de la
  contre-passation du PDF (la seule occurrence restante parle des règles d'affectation) ;
  `Kesh ne lettre encore rien` ×0 ; « toutes ces conditions tiennent » suivi de **six** points ;
  `se modifie|mêmes gardes|modifiables et supprimables|exercice (est|reste) ouvert` sur les deux
  PDF aplatis : chaque occurrence porte la réserve « lettrée » ou est étrangère (compte, règle
  d'affectation, dévalidation de facture) ; contrôle final des catalogues i18n et replis (commande
  d'AC15 ii) : seuls `error-opening-complement-account-moved` et son repli Rust restent sans
  réserve, triés par la fiche ; relevé des cycles (`sans cycle|cycle connu|…`) : chaque site
  inscrit est réécrit, `bank_accounts.rs:717` et `opening_complement.rs:26` étrangers ;
  `DETACHED_SUPPLIER_SETTLEMENT|journal-entries-modify-blocked` : chaque fichier de code porte
  `ENTRY_LETTERED`, sauf `JournalEntryForm.svelte` (repli du seul `FISCAL_YEAR_CLOSED`) et
  `i18n-keys.test.ts` (historique), triés.
- **Mutations** (`/home/gcorbaz/devel/kesh-gate-logs/15-1a-ii-mutations.md`) : **16 rejouées,
  15 tuées**. Backend 13/13 tuées — la paire C117 (marque avant 2-bis), la marque avant le verrou
  de période sur les trois chemins (+ le chemin `false`), `lettering_guard` retiré de
  `modification_blocker`, la marque sous `enforce_ownership`, la marque avant la version, R6 qui
  ne lettre rien, R6 qui réécrit un montant, R6 qui évalue la règle des périodes sur l'origine,
  pas de relecture (C116), seconde branche ignorée. Frontend : F1 et F3 tuées ; **F2 survit**
  (clé `i18nMsg` remplacée, repli FR intact : `i18nMsg` est simulé par son repli dans
  `blocker-messages.test.ts`) — angle mort préexistant, commun aux onze autres branches, non
  propre à cette story. La mutation « R6 ne lettre que le miroir » est inatteignable par la
  primitive (un groupe exige deux lignes du même compte).
- **Gates, au commit de code** (base `kesh_151aii` remise à zéro par `DROP/CREATE` + migrations +
  seed avant le gate ; tmpfs MariaDB 1,3 Go / 8 Go avant) : `scripts/test-fast.sh` (fmt, clippy
  `-D warnings`, nextest profil défaut à huit threads) **3152/3152**, 4 ignorés (référence
  `0724904c` : 3135 ; **+17 tests** recomptés aux deux bornes : `journal_entry_reversal_e2e.rs`
  41 → 53, `journal_entries.rs` (module) 57 → 60, `supplier_invoices_repository.rs` 50 → 51,
  `reconciliation_cancel.rs` 1 → 2, `letterings.rs` 31 → 31 — test étendu) ; Vitest **1152/1152**
  (référence 1149, +3 cas `it.each`) ; `npm run check` 0 erreur (27 avertissements, ceux de
  `main`) ; `lint-i18n-ownership` vert ; **E2E complet** (base `kesh_e2e_151aii` reconstruite, backend sur le port 3017, binaire et frontend buildés au commit de code `81d3e80f`) : **246 passés, 8 échecs, 19 ignorés** — les 7 KF-029 (`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) et `sidebar-navigation.spec.ts:75` (pollution d'état connue), **vert rejoué seul** (1 passed). Aucun KF-045 (run de l'après-midi). tmpfs MariaDB après les gates : 1,4 Go / 8 Go.
- **Tests existants qui changent de forme** (T12, quatre groupes) : tous verts au gate complet —
  `update_waits_for_a_concurrent_reversal_then_refuses`,
  `delete_waits_for_a_concurrent_reversal_then_refuses`,
  `update_and_a_reversal_of_the_same_year_can_deadlock`,
  `settlement_cancellation_is_replayed_when_it_is_the_deadlock_victim`, les quatre « clôture
  attend une annulation », `supprimer_avec` et ses tests,
  `reverse_in_tx_disparait_avec_le_rollback_de_l_appelant`.
- **Choix consignés** au registre : C-15-1a-ii-1 à C-15-1a-ii-6.

### File List

- `CHANGELOG.md`, `README.md`
- `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/journal_entries.rs`
- `crates/kesh-api/tests/audit_route_registry.rs`, `crates/kesh-api/tests/journal_entry_reversal_e2e.rs`
- `crates/kesh-db/src/errors.rs`, `crates/kesh-db/src/repositories/invoices.rs`,
  `crates/kesh-db/src/repositories/journal_entries.rs`
- `crates/kesh-db/tests/journal_entries_modification.rs`, `crates/kesh-db/tests/letterings.rs`,
  `crates/kesh-db/tests/reconciliation_cancel.rs`, `crates/kesh-db/tests/supplier_invoices_repository.rs`
- `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl`
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, `docs/api-external.md`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`,
  `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/admin-manual.pdf`
- `frontend/src/lib/features/journal-entries/{journal-entries.types.ts, journal-entries.api.ts,
  blocker-messages.ts, blocker-messages.test.ts, form-helpers.ts, form-helpers.test.ts,
  JournalEntryForm.edit.test.ts}`
- `frontend/src/lib/shared/{i18n-keys.test.ts, i18n-libelle-en-dur.test.ts}`
- `frontend/src/routes/(app)/journal-entries/[id]/+page.svelte`,
  `frontend/src/routes/(app)/settings/opening-balances/+page.svelte`
- `_bmad-output/implementation-artifacts/{15-1a-ii-gardes-du-lettrage.md, sprint-status.yaml,
  epic-15-choix-autonomes.md}`

## Change Log

### Revue de code P2 ciblée — 2026-10-09 (Opus ; boucle close)

- **Passe ciblée** sur `581040aa` (prompt `f6e9f99e`, rapport `/home/gcorbaz/devel/kesh-gate-logs/15-1a-ii-review-p2-ciblee.md`) :
  **0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW**. Les cinq axes exercés : les autres 409 passés par `refusal_409` gardent leur
  forme ; aucun client ne lit `documentNumber` sur `ENTRY_LETTERED` ; la garde de longueur est levée avant toute pose de
  marque ; `lettering_guard` n'a aucun appelant hors de son module, et ses trois appelants vérifient la société ; le test
  de rollback prouve la marque et l'audit dans la transaction puis leur absence après.
- **LOW laissés en dette écrite** (aucun ne toucherait le code de production) : L1 — les deux assertions d'après le
  rollback ne sont mordues par aucune des trois mutations (la preuve positive dans la transaction suffit ; redondance à
  signaler au commentaire) ; L2 — garde de longueur sans test, branche inatteignable par construction ; L3 — le
  doc-comment du cycle lignes ↔ écriture (`journal_entries.rs:2362`) et la note du Pattern 5 ne nomment que l'acte 1 de
  la création, la dissolution fait le même croisement (même défense : le rejeu) ; L4 — le commentaire de
  `journal_entry_reversal_e2e.rs:269` renvoie à un journal de mutations hors du dépôt.
- **Boucle de revue close** : la passe ciblée de fin de boucle ne demande aucun correctif de production (CLAUDE.md §
  « La passe ciblée »). Trend : P1 (Sonnet ×3) 1 MEDIUM → P2 ciblée (Opus) 0. Dernier commit de code : `581040aa` ; ses
  gates (backend 3161/3161, Vitest 1159/1159, E2E 247 / 7 KF-029) tiennent jusqu'au prochain rebase.

### Revue de code P1 — 2026-10-09 (Sonnet ×3 ; remédiation Opus 5.5)

- **Passe P1** (Sonnet, trois lentilles, diff `0724904c..5a54ec01`) : B 0 MEDIUM / 5 LOW, E 0 MEDIUM / 6 LOW, A 1 MEDIUM / 4 LOW — **1 MEDIUM, 15 LOW** (B-1 = E-2 et B-3 = E-1 convergents : 13 distincts). Rapports : `kesh-gate-logs/15-1a-ii-review-p1-{B,E,A}.md`.
- **Rebase** sur `1ae3963e` (15-6d, #590) avant la remédiation : C-15-1a-ii-11.
- **Remédiation, commit `581040aa`** — **touche du code de production** (`kesh-db` `journal_entries.rs`, `kesh-api` `errors.rs`) :
  - **A1 (MEDIUM)** — `reverse_in_tx_disparait_avec_le_rollback_de_l_appelant` lit, dans la transaction, les marques `reversal` des quatre lignes (deux clés) et les deux audits `lettering.created`, puis, après le rollback, des marques nulles sur l'origine et zéro audit. Mutations M-A1-1 (audit commité sur une autre connexion), M-A1-2 (R6 ne lettre rien), M-A1-3 (`COMMIT` pour l'appelant) **tuées** ; la marque écrite hors de la transaction est **non jouable** (verrou de l'origine, miroir invisible) — C-15-1a-ii-9, journal `15-1a-ii-review-p1-mutations.log`.
  - **B-1 = E-2** — garde de longueur avant le `zip` origine/miroir → `DbError::Invariant` (production).
  - **B-3 = E-1** — `lettering_guard` et `Lecture` privées au module, `_company_id` retiré (production) ; jointure écartée — C-15-1a-ii-8.
  - **B-5** — le 409 `ENTRY_LETTERED` porte `details.letteringCode` (plus `documentId`/`documentNumber`), `refusal_409` commun (production) ; `api-external.md` (trois sites), CHANGELOG, test AC8 au `PUT` et au `DELETE` — C-15-1a-ii-7.
  - **A2** — docstring du test AC9 (f) : ne déclare tuées que M8 et M9, la variante « miroir seul » dite non jouée.
  - **A3** — doc de `update_journal_entry` : quatre cycles (trois hérités + lignes ↔ écriture). Valeur grepée (`(trois|3) cycles`, `cycles? hérités?`) : plus aucun site.
  - **E-3** — `user-manual.tex` (ouverture) : l'exemple de lettrage ne promet plus les encaissements de facture ; PDF régénéré (`make -B user`), phrase vérifiée dans le PDF aplati. Symptôme grepé (`lettre typiquement|encaissements qui suivent|lettrer … encaissement`) : aucun autre site.
  - **E-4** — `api-external.md` : l'écriture lettrée se contre-passe aussi ; le délettrage n'est requis que pour la modifier.
  - **E-5** — cycle lignes ↔ écriture à l'en-tête de `reverse_in_tx_inner` ; ligne `/reverse` (et les quatre annulations) au tableau et aux notes du Pattern 5.
  - **E-6** — commentaire de `letterings.rs` mis au présent.
  - **B-4** — rien à faire : déjà tranché à C131 (7). **B-2, A4, A5** — gardés en dette, motifs à C-15-1a-ii-10 (A4 : issue P3 à ouvrir).
- **Tests** : aucun test neuf (recompté aux deux bornes `7c9a478e` → `581040aa` : `journal_entries.rs` module 60 → 60, `journal_entry_reversal_e2e.rs` 53 → 53) ; deux tests étendus.
- **Gates, au commit de code `581040aa`** (bases `kesh_151aii` et `kesh_e2e_151aii` reconstruites — `DROP/CREATE`, 76 migrations, seed ; tmpfs MariaDB 1,3 Go / 8 Go avant et après) : `scripts/test-fast.sh` **3161/3161**, 4 ignorés (3152 de la story + 9 de la 15-6d) ; Vitest **1159/1159** (1152 + 7 de la 15-6d) ; `npm run check` 0 erreur, 27 avertissements ; `lint-i18n-ownership` vert ; build ; **E2E complet** (port 3017) : **247 passés, 7 échecs, 19 ignorés** — exactement les sept KF-029 (`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`). Journaux : `15-1a-ii-gate-review-p1.log`, `15-1a-ii-fe-review-p1.log`, `15-1a-ii-e2e-review-p1.log`.
- **Suite** : la remédiation touche la production → la boucle n'est pas close ; passe P2 complète (Opus) à lancer.

### Développement — 2026-10-09 (Opus 5.5, bmad-dev-story)

Story développée sur `0724904c` : gel `ENTRY_LETTERED` sur les trois chemins et l'écran (AC8),
contre-passation qui lettre (R6, AC9), audit et invariant (AC10, AC13 part ii), documentation et
textes d'écran (AC15 part ii), les six textes provisoires de la 15-1a-i qui concernent le
`reversal` réécrits au même commit. +17 tests Rust, +3 cas Vitest ; 16 mutations, 15 tuées.
Statut → `review`. Revue de code à lancer (non lancée ici). Choix C-15-1a-ii-1 à 6.

### Reçu de la 15-1a-i — 2026-10-09 (Opus 5.5, remédiation de la revue de code P2 de la 15-1a-i)

Section « Reçu de la 15-1a-i — revue de code P2 » ajoutée (finding A2-1, registre C-15-1a-i-7 et
C-15-1a-i-11) : six textes provisoires écrits par la 15-1a-i (« Kesh ne lettre rien de lui-même »,
origines « réservées »), dont quatre à réécrire ici au même commit que R6 — contradiction avec la phrase
prescrite par AC15 (ii) sinon. Corps de cette fiche non réécrit. Édition hors passe.

### Remédiation de la validation P6 — 2026-10-09 (Opus 5.5)

**Validation P6** (Sonnet ×2, contexte frais, prompt versionné `3acf1860`) : R (chasseur de
régressions, cible `6b4280e2`) **0 HIGH, 1 MEDIUM, 4 LOW** ; F (adversaire plein périmètre) **0 HIGH,
0 MEDIUM, 3 LOW** (`target/gate-logs/15-1a-ii-p6-{R,F}.md`). Aucun recoupement. **Distincts : 0 HIGH,
1 MEDIUM, 7 LOW.** **Trend** (15-1a puis 15-1a-ii) : P1 **2 HIGH / 15 MEDIUM** → P2 **0 / 5 / 16 LOW**
→ P3 **0 / 4 / 16** → P4 **0 / 4 / 12** → P5 **0 / 3 / 10** → P6 **0 / 1 / 7**. Modèles : P1 Opus ×2,
P2 Sonnet ×2, P3 Opus ×2, P4 Sonnet ×2, P5 Opus ×2, P6 Sonnet ×2 (rotation D6) ; remédiation Opus 5.5.

**Nature** : **aucune règle ne change** — ni le rang d'`ENTRY_LETTERED` (C126), ni R6 (C97, C129), ni
la défense par le rejeu, ni C132 (seulement une réserve de lecture). Le MEDIUM R6-1 est le **résidu
d'un correctif de P5** (F5-2 propagé par la formulation et non par la valeur) — le motif mesuré de la
§ « Propagation post-patch » ; les LOW sont des précisions d'exécution. Signal D5 : la sévérité
**décroît** (3 → 1 MEDIUM) ; pas de recyclage de règle. Aucun choix neuf au registre.

| finding | décision | où |
|---|---|---|
| R6-1 (MEDIUM) | `audit_route_registry.rs:80-83` (module-doc, « sans cycle connu ») et `:200-202` (« par uniformité avec le `PUT` ») inscrits au tableau d'AC8 ; **relevé par la valeur** écrit (`sans cycle\|cycle connu\|par uniformité\|No known cycle\|by uniformity\|known cycle`), rejoué sur `ec745d0c` : **8 lignes**, 6 inscrites (`routes/journal_entries.rs:724`, `:725` ; `audit_route_registry.rs:82`, `:201` ; `journal_entries.rs:1218` ; Pattern 5 `:332`), 2 étrangères triées (`audit_route_registry.rs:95`, candidates de l'exception (iv), « sans cycle démontré » ; `opening_complement.rs:26`, règle du complément d'ouverture) | AC8 (tableau, relevé), T4, Dev Notes |
| R6-2 (LOW) | doc de `reverseJournalEntry` (`journal-entries.api.ts:42-45`, « Ne modifie rien … L'origine demeure ») réécrite selon C129 ; « quatre endroits » → **cinq** ; `ne modifie rien\|demeure` ajoutés au relevé de R6 (« demeure » rend aussi les « mises en demeure », triées), `ne modifie rien` au contrôle final d'AC15 ii | R6 (tableau, relevé), T5, AC15 ii |
| R6-3 (LOW) | réserve sur C132 : « trois » et « huit » sont les chiffres de `ec745d0c` ; après la 15-12b (ordre préféré C112), ils se relisent sur le `main` du moment — la règle (nommer la marque, ne pas la compter) vaut quel que soit le total | AC8 (décompte) |
| R6-4 (LOW) | T12 gagne un quatrième groupe de tests existants relus : `supprimer_avec` (`journal_entries.rs:3202`, appel `reverse_in_tx` `:3219`) et `reverse_in_tx_disparait_avec_le_rollback_de_l_appelant` (`:4856`, appel `:4891`) | T12 |
| R6-5 (LOW) | « les six tests nommés (a) à (e) » → (a) à (e) **et** `reconciliation_cancel_letters_the_reversal_pair` | T12 |
| F6-1 (LOW) | (c) gagne un second cas : origine datée ≤ `books_locked_through` → `201`, groupe `reversal` posé ; mutation nommée (règle des périodes évaluée sur l'origine) | AC9 (c), T12 |
| F6-2 (LOW) | le cinquième `\item` (`user-manual.tex:506`) passe de `.` à `~;` | AC15 ii, T11 |
| F6-3 (LOW) | `\| 'ENTRY_LETTERED'` **en dernier** dans l'union `ModificationBlocker` (après `'PERIOD_LOCKED'`, `:96`) et dans `ATTENDU` — ordre de précédence que le doc `:85` affirme | T4-bis |

**Revérification** (sur `origin/main` = `ec745d0c`) : `audit_route_registry.rs:82`, `:201`, `:95` ;
`opening_complement.rs:26` ; `journal-entries.api.ts:42-45` (`:43` « Ne modifie rien », `:44`
« demeure ») ; `journal_entries.rs` `supprimer_avec` déclaré à **`:3202`** (R citait la plage
`:3195-3225`), appels `:3219`, `:4891`, test `:4856` ; `user-manual.tex:506` (cinquième `\item`, point
final) ; `journal-entries.types.ts:85`, `:91`, `:96` ; `blocker-messages.test.ts:24-37` (`ATTENDU`,
`PERIOD_LOCKED` en dernier). `ReversalBlocker` (`journal-entries.types.ts:46-54`) ne compte pas de
verrou de période : la contre-passation d'une origine sous la borne est bien légitime (F6-1).

**Recompte de cette fiche** (corps, avant `## Change Log`) : **R6** ; **5 critères** ; **7 tâches** ;
**1 clé i18n** neuve, **3** réécrites (inchangé) ; **13 tests neufs nommés** (inchangé : F6-1 étend
(c)) ; mutations nommées **9** (8 en P5, plus celle de (c) — F6-1) ; endroits de la doctrine « n'en
modifie aucune » réécrits : **5** ; modules : **onze** (inchangé).

### Remédiation de la validation P5 — 2026-10-09 (Opus 5.5)

**Validation P5** (Opus 5.5 ×2, contexte frais, prompt versionné `c6a88f03`) : R (chasseur de
régressions, cible `329ab812`) **0 HIGH, 1 MEDIUM, 8 LOW** ; F (adversaire plein périmètre) **0 HIGH,
2 MEDIUM, 9 LOW** (`target/gate-logs/15-1a-ii-p5-{R,F}.md`). Recoupements : R5-4 = F5-2 (LOW chez R,
MEDIUM chez F : **MEDIUM retenu**), R5-2 = L-5, R5-3 ⊇ L-2, R5-5 ⊇ L-1, R5-6 = L-3, R5-7 = L-7, R5-9 ⊇
L-9. **Distincts : 0 HIGH, 3 MEDIUM** (R5-1, F5-1, F5-2) **et 10 LOW** (R5-2, R5-3, R5-5 à R5-9, L-4, L-6,
L-8). **Trend** (15-1a puis 15-1a-ii) : P1 **2 HIGH / 15 MEDIUM** → P2 **0 / 5 / 16 LOW** → P3 **0 / 4 /
16** → P4 **0 / 4 / 12** → P5 **0 / 3 / 10**. Modèles : P1 Opus ×2, P2 Sonnet ×2, P3 Opus ×2, P4 Sonnet
×2, P5 Opus ×2 (rotation D6) ; remédiation Opus 5.5.

**Nature** : tous sont des défauts d'**inventaire ou de propagation** ; **aucune règle ne change** —
ni le rang d'`ENTRY_LETTERED` (C126), ni R6 (C97, C129), ni la défense par le rejeu (F-4 de P4). R5-1
est le résidu d'un correctif de P4 (F-2, propagé aux manuels et non aux catalogues) ; F5-1 et F5-2
sont des omissions d'**origine** (la phrase du manuel depuis C129, les textes du cycle depuis la
rédaction d'AC8 en P4). Signal D5 : la sévérité **décroît** (4 → 3 MEDIUM) ; pas de recyclage de règle.

| finding | décision | où |
|---|---|---|
| R5-1 (MEDIUM) | `opening-balances-complete-confirm` et `opening-balances-locked-already-has-entries` (quatre locales, replis `settings/opening-balances/+page.svelte:233-234`, `:386-388`) reçoivent la réserve « lettrée », textes arrêtés en quatre langues ; `error-opening-complement-account-moved` trié ; **contrôle final étendu aux catalogues i18n et aux replis** | AC15 ii (tableau, textes, contrôle final), T10 ii, T11, Dev Notes (onzième module) |
| F5-1 (MEDIUM) | `user-manual.tex:608` (« L'écriture d'origine n'est pas touchée ») → doctrine C129, et la section dit que la contre-passation **lettre** ; « n'est pas touchée », « ne touche pas », « inchangée » ajoutés au relevé ; `pas touchée` au contrôle du PDF aplati | R6 (tableau, relevé), T5, T11, AC15 ii |
| F5-2 = R5-4 (MEDIUM) | Le cycle lettrage (ligne puis en-tête) ↔ `PUT`/`DELETE` (en-tête puis lignes) **écrit** partout où il était nié ou l'ordre incomplet : Pattern 5 `:326`, `:327`, `:331`, `:332` ; doc de `delete_journal_entry` `:723-728` ; « # Ordre des verrous » et cycles d'`update` `:1205-1209`, `:1218-1234` ; « # Sérialisation » de `delete_in_tx` `:1619-1630` ; commentaire au verrou des lignes de l'origine (R6, `:2266`) — pas de ligne Pattern 5 pour `reverse` (hors périmètre, comme C131 point 6) ; rejeu du `DELETE` `:736` cité | AC8 (tableau neuf), T4 |

**LOW appliqués** : R5-2 = L-5 (`admin-manual.tex:2128`) ; R5-3 = L-2 (`kesh-db/src/errors.rs:152-153`,
`:161`, `:172`, `:216-217`, `blocker-messages.ts:76-79`) ; R5-5 = L-1 (cadre « aucune ligne lettrée » :
`routes/journal_entries.rs:711-714`, `journal_entries.rs:25-27`, `:44-48`, `:1161-1164`, `:1552-1555`,
`journal-entries.api.ts:69-71`) ; R5-6 = L-3 (textes du test renommé `:257-258`, `:281-283`, `:303`,
`:318` — ⚠️ L-3 citait `:308`, le message « ne doit pas être touchée » est à **`:303`**, relu) ; R5-7 =
L-7 (**C132** : les totaux « huit » comptent les refus atteignables et restent ; `invoices.rs:1471-1473`
et `kesh-db/src/errors.rs:241-244` gardent « trois » et **nomment** la marque, inatteignable — la
décision « → quatre » de P4 est retirée) ; R5-8 (« six lectures », `:1073`) ; R5-9 (`:2110` → `:2108` ;
chaîne de la 15-12b à `:126-128` ; décompte des mutations, ci-dessous ; `{code}` → `{key}`) ; L-4
(`README.md:220` trié, v0.12.0 publiée) ; L-6 (`api-external.md:251`, une phrase) ; L-8 (apostrophe
typographique du message neuf ; le repli Rust suit son voisin, `errors.rs:2832`).

**Revérification** : chaque numéro neuf relu sur `origin/main` = **`ec745d0c`** (le merge de la 15-7a1
ne touche aucun fichier cité : `git diff --stat 5e4bec50 ec745d0c` filtré). Écarts aux rapports
relevés : `errors.rs:1252` (R) → **`:1253`** ; `:308` (F) → **`:303`** ; bornes d'`update`
`:1205-1216` (F) → **`:1205-1209`** (ordre) et **`:1218-1234`** (cycles) ; `:1162-1165`/`:1552-1556`
→ **`:1161-1164`**/**`:1552-1555`**.

**Propagation** (grep par la valeur, sur `origin/main`, `crates frontend/src docs README.md
CHANGELOG.md website`, PDF des deux manuels aplatis) : `pas touchée|n'en modifie aucune|intacte|No known
cycle|pas pour un cycle connu|modifiable tant que|reste modifiable|tant que l'exercice|untouched|
unverändert|resta intatta|remains editable|bleibt … änderbar|Resta modificabile` — sites neufs tous
inscrits ou triés ; en plus des rapports, **`journal_entries.rs:25-27`** (doc du module, « tant que
l'exercice est ouvert, par modification tracée ») est trouvé et inscrit, et `docs/kesh-specifications.txt`
(FR23, FR86) trié (exigences du PRD). Les « intacte » restants (import, contacts, factures, rapports)
sont étrangers à la contre-passation. PDF : `n’est pas touchée` ×1 (user, section de la
contre-passation) ; « huit refus » / « huit motifs » ×1 chacun (admin) — triés par C132.

**Recompte de cette fiche** (`grep` sur le corps, avant `## Change Log`) : **R6** ; **5 critères** ;
**7 tâches** ; **1 clé i18n** neuve, plus **3 clés existantes réécrites** (1 en P4, C129 ; 2 en P5, R5-1)
; **13 tests neufs nommés** (inchangé) ; mutations nommées **8** — et non 7 comme l'écrivait P4 : la
paire C117 (1), une par chemin de précédence (3), retirer `lettering_guard` de `modification_blocker`
(1), et **trois** au test (f) (« ne lettre que le miroir », « ne lettre rien », « réécrirait un montant
ou un compte » — P4 avait compté les deux premières pour une).

### Remédiation de la validation P4 — 2026-10-09 (Opus 5.5)

**Validation P4** (Sonnet 5.5 ×2, contexte frais, prompt versionné `b53f7278`) : R (chasseur de
régressions) **0 HIGH, 1 MEDIUM, 8 LOW** ; F (adversaire plein périmètre) **0 HIGH, 3 MEDIUM, 6 LOW**
(`target/gate-logs/15-1a-ii-p4-{R,F}.md`). Recoupements : R4-6 = F-5, R4-9 = F-8 ; R4-1 = R4-2 de la
15-1a-i (traité là, C128). **Distincts : 0 HIGH, 4 MEDIUM** (R4-1, F-1, F-2, F-3) **et 12 LOW** (R4-2 à
R4-9, F-4, F-6, F-7, F-9). **Trend** (15-1a puis 15-1a-ii) : P1 **2 HIGH / 15 MEDIUM** → P2 **0 / 5 /
16 LOW** → P3 **0 / 4 / 16** → P4 **0 / 4 / 12**. Modèles : P1 Opus ×2, P2 Sonnet ×2, P3 Opus ×2, P4
Sonnet ×2 (rotation D6) ; remédiation Opus 5.5.

⚠️ **Signal D5, déclaré au Project Lead** : la sévérité **stagne** (P3 4 MEDIUM → P4 4 MEDIUM). Mais trois
des quatre sont des défauts d'**origine**, présents depuis P1 et jamais vus — F-1 (test de
correspondance muet), F-2 (manuel d'administration et README hors du relevé), F-3 (R6 marque l'origine
que trois textes disent intacte) — ; un seul, R4-1, **naît d'un correctif** (C127). Au sens de
l'amendement D5, la revue trouve des défauts neufs plutôt qu'elle ne tourne en rond : pas de découpage
proposé. **Signal de périmètre** *(F-6)* : dix modules métier, seuil franchi, déclaré (Dev Notes).

| finding | décision | où |
|---|---|---|
| R4-1 (MEDIUM) = R4-2 de la 15-1a-i | Nom des exercices en mode `System` par lecture ordinaire non verrouillante (15-1a-i, R7 point 3) ; coût de R6 recompté | AC10 ii, R6 « Coût », C128 |
| F-1 (MEDIUM) | `each_screen_code_maps_to_its_put_and_delete_refusal` étendu à douze codes (cas `ENTRY_LETTERED`, `vus.len() == 12`, docstring) ; mutation nommée | AC8 (liste des sites), T4, T12 |
| F-2 (MEDIUM) | `admin-manual.tex:1920`, `:1959`, `:2093` et `README.md:29` ajoutés au relevé ; les deux PDF contrôlés aplatis | AC15 ii, T11 |
| F-3 (MEDIUM) | Décision de l'orchestrateur : origine intacte dans ses montants, comptes, dates et libellés, seules ses lignes reçoivent la marque ; doc-comment `:477`, clé du dialogue (quatre locales, texte arrêté), repli `:475` ; test renommé et étendu, mutations nommées | R6, AC9 (f), T5, T10 ii, T12, C129 ; 15-1a2 Reçu point 20 |

**LOW appliqués** : R4-2 (doc-comments `kesh-db/src/errors.rs:121-122`, `:189-191`, `:199`,
`journal_entries.rs:1640-1641`, `invoices.rs:1471-1473`, `:1650-1657`), R4-3 (doc du label : route
`:168-169`, type `:77-80`, `api-external.md:229`), R4-4 (phrase `:509-513` du manuel ; « aboutit toujours »
ramené à « hors concurrence »), R4-5 (liste des tests concurrents corrigée et nommée), R4-6 = F-5 (la
15-12b est réalignée depuis `d0161d96` : mention périmée réécrite), R4-7 (`blocker-messages.test.ts:24`,
`:50`, `:52`, `:53`), R4-8 (tableaux d'`api-external.md` `:233-249`, `:267-277` ; noms `snake_case`
d'AC9 et de la paire C117 ; **réfuté en partie** : `create_in_tx_inner` renumérote bien à `:446-447`
— `:446` `for (idx, line)`, `:447` `let line_order = (idx as i32) + 1`, relus sur `5e4bec50` —, la fiche
était exacte), R4-9 = F-8 (fenêtre sans écran de délettrage écrite, jamais publiée — C131), F-4 (AC8 :
ordres opposés, la défense est le rejeu), F-6 (modules recomptés), F-7 (une phrase à `api-external.md:255` ;
`invoices.rs:1471-1473` et `kesh-db/src/errors.rs:241-244` comptent un quatrième motif), F-9
(`is_letterable_account` nommée en R6 et T5).

**Propagation** (grep par la valeur) : `onze|eleven` sur `crates`, `frontend/src`, `docs` — les sites du
douzième code sont tous nommés (`routes/journal_entries.rs:163`, `kesh-db/src/errors.rs:206`,
`journal_entry_reversal_e2e.rs:2102/2219`, `journal-entries.types.ts:85`, `blocker-messages.test.ts:24/50`,
`i18n-libelle-en-dur.test.ts:163`), les autres « onze » (tables d'export de la 25-5-a, chemins de
création, clés i18n historiques) sont étrangers ; `n'en modifie aucune|reste intacte|origine
intacte|garde l'origine|untouched|unverändert|resta intatta` — trié à R6 ; `se modifie|mêmes
gardes|modifiables et supprimables` sur les deux manuels et le README — relevé d'AC15 ii ;
`3-ter-bis` — ne subsiste qu'aux Change Logs (historique).

**Recompte de cette fiche** (`grep` sur le corps, avant `## Change Log`) : **R6** ; **5 critères** ;
**7 tâches** ; **1 clé i18n** neuve (plus **1 clé existante réécrite**, C129) ; **13 tests neufs
nommés** (6 en P3, + 6 noms d'AC9 et le nom de la paire C117 — tests déjà décrits, désormais nommés),
plus deux tests existants modifiés (`each_screen_code_maps_to_its_put_and_delete_refusal`, et
`reverse_creates_the_opposite_entry_and_leaves_the_origin_intact` renommé) ; mutations nommées **7**
(4 de P3, + « retirer `lettering_guard` de `modification_blocker` », + deux sur R6 au test (f)).

### Création au découpage de la 15-1a, et remédiation de la validation P3 — 2026-10-09 (Opus 5.5)

**Validation P3 de la 15-1a** (Opus ×2, contexte frais) : **0 HIGH, 4 MEDIUM, 16 LOW distincts**
(`target/gate-logs/15-1a-p3-{R,F}.md` ; ventilation au Change Log de la 15-1a-i). **Trend de la 15-1a** :
P1 **2 HIGH / 15 MEDIUM** → P2 **0 HIGH / 5 MEDIUM / 16 LOW** → P3 **0 HIGH / 4 MEDIUM / 16 LOW**. Modèles : P1 Opus ×2, P2 Sonnet ×2, P3 Opus ×2 (rotation D6) ; remédiation Opus 5.5.
⛔ **Signal D5 — recyclage** : R3-1, R3-2, R3-3 naissent de correctifs de P2 sur des règles métier ;
déclencheur de C118 atteint, découpage décidé par l'orchestrateur (C124). Cette fiche porte la part
(ii).

| finding | décision | où (cette fiche) |
|---|---|---|
| R3-1 = F3-1 | `ENTRY_LETTERED` parle **après** tout refus que le délettrage ne peut lever : `delete_in_tx` 3-quinquies, `update_in_tx` 7-bis, `modification_blocker` dernier ; doc-comments réécrits (dont `unvalidate`, L6) ; tests par paire sur chaque chemin, mutation « permuter » ; le message tient sa promesse (test) | AC8, AC15 (ii), T4, T12, C126 |
| F3-2 (part ii) | CHANGELOG : entrées #532 réécrites (`ENTRY_LETTERED` en dernier, « écriture lettrée » à part, condition de la suppression) ; *Modifié* : la contre-passation lettre (`201`) | AC15 (ii), T11, C127 |

**LOW appliqués ici** : L3 (`routes/journal_entries.rs:163-166`), L6 (doc-comment d'`unvalidate`), L8
(`kesh-api/src/errors.rs:2822`, `invoices.rs:1660`/`:1530-1607`, FAQ `:2224`), L9
(`i18n-libelle-en-dur.test.ts:160-165`, cinq branches propres), F3-3 (sixième `\item` du manuel), F3-6
(motifs du test `POST` × `PUT`). Axe que F déclarait non exercé, repris en tâche : les tests qui
provoquent un interblocage avec une contre-passation (T12). **L5** (la 15-12a décrit l'ancien verrou de
la 15-1a, `15-12a-cloture-dans-l-ordre.md:94-108`) et la **15-12b** (étape « 3-ter-bis », chaîne de
`unvalidate`) sont portés à l'orchestrateur : fiches 15-12* non modifiées ici.

**Recompte de cette fiche** (`grep` sur le corps, avant `## Change Log`) : **R6** ; **5 critères** (AC8,
AC9, AC10 part ii, AC13 part ii, AC15 part ii) ; **7 tâches** (T0 part ii, T4, T4-bis, T5, T10 part ii,
T11 part ii, T12 part ii) ; **1 clé i18n** (T10 part ii).

- **2026-10-09 — validation P7 ciblée (Haiku, prompt `15-1a-ii-validate-prompt-p7-ciblee.md`)** : rapport
  `target/gate-logs/15-1a-ii-p7-ciblee.md`, 2 MEDIUM + 3 LOW annoncés. **Les deux MEDIUM et le LOW F6-2 sont
  écartés comme erreurs de catégorie**, vérifiées par l'orchestrateur (`grep -nF`) : la lentille a constaté
  dans le CODE de `origin/main` l'absence de ce que la fiche PRESCRIT — la doc de `reverseJournalEntry`
  (texte arrêté au tableau de R6, `:93`, repris en T5 `:614`), `'ENTRY_LETTERED'` en dernier dans l'union
  `ModificationBlocker` (T4-bis `:594-595`, tableau `:791`) et le `~;` du cinquième `\item` (`:539`, `:641`) —
  malgré l'avertissement explicite du prompt. Ses autres axes sont exacts et confirment la remédiation P6 : le
  grep élargi d'AC8 rend bien 8 lignes (6 inscrites, 2 étrangères, tri juste), les numéros `:3202`, `:3219`,
  `:4856`, `:4891` sont exacts, les recomptes (5 AC, 7 tâches, 13 tests, 9 mutations, 11 modules) aussi.
  **Bilan P7 : 0 au-dessus de LOW — validation close.** Trend (15-1a puis 15-1a-ii) : P1 2 HIGH / 15 MEDIUM →
  P2 5 → P3 4 (découpage, C124) → P4 4 → P5 3 → P6 1 → P7 ciblée 0. Modèles : Opus ×2, Sonnet ×2, Opus ×2,
  Sonnet ×2, Opus ×2, Sonnet ×2, Haiku (ciblée). Développement après la 15-1a-i.

- **2026-10-09 — reçu de la 15-12b (développée, `adde3bc4`)** : la 15-12b ajoute `LATER_FISCAL_YEAR_CLOSED` aux
  refus de la dévalidation ; les totaux passent de « huit » à « **neuf** » (`invoices.rs:1376`,
  `invoices/[id]/+page.svelte:355`, `admin-manual.tex:1922`, `:1964`, sur la branche de la 15-12b). C132
  raisonne sur « huit » : au T0, relire ces totaux sur le `main` du moment (réserve R6-3 déjà écrite) et
  nommer la marque sans la compter. Édition de l'orchestrateur, pas de passe.
