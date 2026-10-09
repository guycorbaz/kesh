# Story 15.1c-ii : Le lettrage dans le reste de Kesh — fiche d'écriture, Grand livre, manuel

## Status

ready-for-dev **après la livraison de la 15-1c-i** — créée le 2026-10-09 par le découpage de la 15-1c à la
remédiation de sa validation P1 (registre **C-15-1c-1**) ; validation P2 remédiée le 2026-10-09 ; **validation P3
due** (passe complète, avec la 15-1c-0 et la 15-1c-i).

⛔ **Ordre** : **… → 15-1b-0 → 15-1b → 15-1c-0 → 15-1c-i → 15-1c-ii**. Cette story suppose la 15-1c-i mergée : l'écran
`/open-items`, l'état d'URL `?group=` (cible de tous les liens ci-dessous), le champ `letterable` du type
`AccountResponse`, le composant de lien de code. Elle suppose aussi la 15-1b (`LedgerLine.letteringCode`, AC6 de
la 15-1b) et les 15-1a2-* (le manuel qu'elle complète décrit les groupes `document`). **Dernière story du lettrage :
la PR porte `closes #518`** (C-15-1a-ii-3, reporté de la 15-1c à cette moitié par C-15-1c-1).

⛔ **Pas de tag v0.13.0 entre la 15-1c-i et cette story** (C124, étendue par C-15-1c-1) : c'est elle qui retire du
manuel, du CHANGELOG et d'`api-external.md` les phrases « par l'API dans cette version » et « l'écran viendra ».

## Story

**As a** indépendant, PME ou fiduciaire,
**I want** voir le lettrage d'une ligne là où je la lis — sur la fiche d'écriture et au Grand livre —, atteindre
d'un clic les postes ouverts d'un compte et le groupe d'une ligne, et trouver au manuel ce qu'est le lettrage et ce
qu'il ne fait pas,
**so that** le lettrage fasse partie de Kesh, et non d'un écran à part.

Dernière part de la 15-1c (#518), après la 15-1c-0 (serveur) et la 15-1c-i (écran). Story **frontend +
documentation** ; aucune route, aucune migration, aucun code
serveur.

## Critères d'acceptation

*Numérotation de la 15-1c conservée (AC9, AC11 part ii, AC12, AC13 part ii, AC14) ; AC17 est neuf.*

**AC9 — Le code visible ailleurs.**

- **Fiche d'écriture** (`frontend/src/routes/(app)/journal-entries/[id]/+page.svelte`) : une colonne
  **« Lettrage »**, **la dernière** du tableau des lignes — et la ligne *Total* du pied gagne sa cellule (sur
  `f9b6b199`, `colspan={hasProjects ? 2 : 1}` + débit + crédit ; validation P3, R L-5 = F-1) : pour chaque `<tr>`,
  la somme des `colspan` égale le nombre d'en-têtes, avec et sans projets ; une ligne lettrée y montre son code (`JournalEntryLineResponse.letteringCode`, livré par la
  15-1a-i) **en lien** vers `/open-items?group=<code>` (AC6 de la 15-1c-i) ; une ligne ouverte, rien. L'en-tête est
  traduit (`i18nMsg`, clé `journal-entries-*`), même si les en-têtes voisins sont encore en dur (« Compte »,
  « Débit », hors de cette story).
- **Le refus `ENTRY_LETTERED`** (déjà câblé : `blocker-messages.ts`, `form-helpers.ts` → `stale`, motif d'écran
  suffixé du code par `modificationMessage`) gagne **le lien vers le groupe dans le motif d'écran de la fiche** —
  le code y devient cliquable — et **nulle part ailleurs** (C-15-1c-9) : le refus d'un `PUT` ou d'un `DELETE` reste
  un toast (texte), puis la fiche se relit (comportement existant) et c'est son motif qui porte le lien. Le
  « Supprimer » et le « Modifier » n'étant offerts que si `entry.modifiable`, le toast n'arrive qu'en course ;
  `details.letteringCode` n'est pas lu par l'écran.
- **Grand livre** (`frontend/src/lib/features/reports/GeneralLedgerView.svelte`) : une colonne « Lettrage » (code en
  lien vers le groupe), lue dans `LedgerLine.letteringCode` (15-1b AC6) — le type `LedgerLine` de
  `reports.types.ts` gagne `letteringCode: string | null`. **Position fixée** (validation P2, F2-6 ; C-15-1c-20) :
  **la dernière**, après « Solde progressif » — si bien que les colonnes « Débit » et « Crédit » ne bougent pas et
  que les totaux du pied restent dessous. Les **`colspan` codés en dur** — sur `f9b6b199`, 8 en-têtes ; ouverture,
  rupture d'exercice et clôture `colspan="7"` + une cellule ; ligne vide `colspan="8"` ; total des mouvements du
  pied `colspan="5"` + débit + crédit + une cellule — deviennent **calculés** depuis une seule constante (le nombre
  de colonnes), et chaque ligne gagne la cellule de la colonne neuve : pour **chaque** `<tr>` du corps et du pied,
  la somme des `colspan` égale le nombre d'en-têtes.
- **Lien « Postes ouverts de ce compte »** sur l'en-tête d'une section du Grand livre **si et seulement si** le
  compte est lettrable — lu dans la liste des comptes que la page des rapports charge **déjà**, archivés compris
  (`fetchAccounts(true)`, `routes/(app)/reports/+page.svelte`), par `accountId` → `letterable` (C-15-1c-8 ;
  `kesh-report` ne voit pas la lettrabilité, C-15-1b-8, et `LedgerSection` reste inchangée). Cible :
  `/open-items?accountId=<id>&asOf=<fin de la période du Grand livre>` — les postes ouverts **au dernier jour
  affiché**, dont le total égale la clôture de la section **au signe près** : le Grand livre la montre du côté
  naturel du compte (crédit positif pour un passif), l'écran des postes ouverts en valeur absolue suivie de
  « débiteur » ou « créditeur » (AC8 de la 15-1c-i, C-15-1c-16). Si la liste des comptes n'a pas pu
  être chargée : pas de lien (jamais un lien vers un compte non lettrable, qui rendrait 409).
- **Exports** : le CSV et le PDF du Grand livre **ne portent pas** le code (C-15-1b-6, maintenu : C-15-1c-8) ; le
  manuel le dit (AC12).
- Le composant de lien de code est celui de la 15-1c-i (aucune clé i18n) ; les libellés de colonne et de lien sont
  des clés du dossier qui les emploie (`reports-*` dans `features/reports`, `journal-entries-*` sur la fiche).

**AC11 (part ii) — i18n.** Les clés d'AC9 dans les **quatre** locales, au vocabulaire de C-15-1a-i-4 /
C-15-1a-ii-2 (*Ausgleich*, *matching*, *abbinamento*) ; mêmes gardes que la 15-1c-i (AC11 part i : bornes de
`i18n-keys.test.ts` relevées aux deux bornes, `i18n-un-repli-par-cle`, `i18n-libelle-en-dur`, G13
`i18n-repli-divergent-actif`, `i18n-entrees-a-variables` (validation P3, F-7), `e2e-selecteurs-traduits`,
`parity_between_locales`, G9, G4-bis) ; **gate backend
complet** au dernier commit de code (les FTL sont dans `kesh-i18n`).

**AC12 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`).

1. **Une section « Lettrage et postes ouverts »** (une `\section` après « Réconciliation bancaire »), qui dit :
   - ce qu'est un lettrage — un groupe de lignes d'**un** compte d'actif ou de passif, à somme **exactement**
     nulle, de 2 à 200 lignes, son code ;
   - les trois origines et **qui lettre quoi** — les pièces d'office (facture client soldée, facture fournisseur
     payée, 15-1a2), la contre-passation d'office, le reste à la main ;
   - **la règle des périodes** telle qu'elle est en vigueur — **C105** (qui révise **C94**) et la 15-1a R7 : lettrer
     **comme** délettrer exigent qu'**au moins une** ligne soit en période ouverte (exercice ouvert, aucun exercice
     postérieur clôturé, date après le verrou de période) ; un groupe tout entier en période close ne se crée ni ne
     se défait. *(La rédaction précédente renvoyait à « la borne d'exercice (C94) » : la règle de C94 — délettrage
     seul gardé — est périmée.)* Pour l'annulation d'un règlement, d'un solde, d'un rapprochement ou d'un paiement
     fournisseur dont le lettrage est figé : **renvoi** à ce que les sections de ces gestes disent déjà (refus du
     rang 2 bis, 15-1a2-0, documenté par la 15-1a2-i AC18), sans le redire (point 9) ;
   - **les postes ouverts à une date** — l'invariant (total des ouverts = solde du compte à la date = ce que la
     Balance montre pour ce compte, la Balance le donnant du côté naturel du compte et l'écran avec son sens,
     « débiteur » ou « créditeur ») ; **« au X » n'est pas un instantané** (point 8) : la liste d'une date en
     période close (exercice clôturé, ou au plus tard la borne du verrou) se reproduit **tant qu'aucun
     administrateur ne déverrouille la période, ne rouvre un exercice ni ne restaure une sauvegarde** ; au-dessus,
     un délettrage ou l'annulation d'un règlement peut y faire réapparaître des lignes ;
   - les motifs (`reason`, état de la pièce **d'aujourd'hui**), dont la pièce soldée sans lettrage (« rien à faire
     ici ») ;
   - les **propositions** — des paires, aujourd'hui, jamais lettrées sans clic ; la contre-passation en tête ; les
     lignes d'un règlement annulé puis délettré, sans pièce, qui reviennent proposées (point 10) ;
   - **le délettrage** — à l'écran (un groupe s'ouvre par son code depuis la liste, la fiche d'écriture, le Grand
     livre) ; refusé pour un groupe d'une pièce — **il suit la pièce et ses règlements** (texte de la 15-1a2-0 D5 ;
     renvoi aux sections du règlement, de l'avoir et de la facture fournisseur, jamais « annulez le règlement »,
     faux pour un groupe facture + avoir : validation P2, R M-4, C-15-1c-18) — et pour une paire de
     contre-passation dont une ligne reste celle d'une pièce (l'annulation d'une facture fournisseur non payée) ;
   - **ce que le lettrage ne fait pas** — pas de lettrage partiel, pas de tolérance de montant (le solde du reste
     pour une facture, une écriture d'ajustement sinon) ; une facture payée par écriture manuelle se règle **sur la
     facture** ; le code n'est **pas** dans les exports CSV/PDF du Grand livre ;
   - **la frontière avec la réconciliation** (les comptes bancaires ne se lettrent pas ; le rapprochement des
     relevés se fait dans *Mensuel → Réconciliation*) ; **les rôles** (Consultation lit, ne lettre ni ne délettre).
2. **Propagation — par inventaire, non par liste de lignes** (R-8 = F-9 ; § « Inventorier les sites NON
   RÉSOLUS ») : les numéros de ligne cités par la 15-1c étaient tous périmés au 2026-10-09, et trois fiches en
   donnaient trois pour le même passage. Au T0 :
   ```sh
   grep -n -i 'lettr' docs/manual/fr/*.tex
   grep -n -i "par l.API\|viendra\|ne lettre pas encore\|ne lettre rien\|pas encore de lui-même\|backlog" \
     docs/manual/fr/*.tex
   ```
   — **les trois sources des trois PDF** (`user-manual.tex`, `admin-manual.tex`, `marketing-brochure.tex` :
   validation P2, R M-5 = F2-5 ; C-15-1c-19) — et **chaque** site est classé au Dev Agent Record — **réécrit** (et
   comment) ou **juste en l'état** (et pourquoi) ; total des sites = réécrits + justes. Sur `f9b6b199`, **30** lignes
   contiennent « lettr » : 24 du manuel utilisateur, **5** du manuel d'administration (`:1238`, « lettres », faux
   positif à classer juste ; quatre sur la modification et la correction d'une écriture lettrée, qui gagnent à dire
   **où** se fait le délettrage — validation P3, R L-6 = F-9) et **une** de la brochure (`marketing-brochure.tex`,
   bloc « Backlog (Epic 13 à 15) » : « Justificatifs, lettrage, journaux personnalisables. ») ; les 15-1a2-* en
   auront ajouté. **Sites à réécrire certainement**, désignés par leur section et leur
   phrase : *Modifier ou supprimer une écriture*, sixième condition — « (le délettrage se fait par l'API dans cette
   version) » à retirer, et renvoi à la section neuve ; le **glossaire**, entrée *Lettrage* — « Dans cette version,
   le lettrage manuel et le délettrage se font par l'API (`/api/v1/letterings`) ; l'écran viendra. » à retirer, et
   renvoi ; *Grand livre* — la colonne, le lien, l'absence du code aux exports ; et tout « délettrez-la d'abord »
   qui gagne à dire **où** (l'écran, ou le lien du motif de la fiche). **Sites à relire** : *Le verrou de période*
   (le lettrage figé), *Corriger une écriture : la contre-passation* (lettrage d'office), *Enregistrer et annuler un
   règlement*, *Avoirs et notes de crédit*, *Factures fournisseurs et paiements*, *Exercices* (réouverture), *Soldes
   de départ* (« les encaissements des factures ne se lettrent pas encore » — vrai sur `e892dcfa`, faux après la
   15-1a2-i si elle ne l'a pas retiré), *Balance*.
3. **Preuves** (validation P2, R L-5 = F2-L7 ; C-15-1c-23). La preuve **négative** est le `grep` des sources
   `.tex`, les deux apostrophes couvertes par un point : `grep -n -i "l.écran viendra\|par l.API dans cette
   version\|ne se lettrent pas encore\|pas encore de lui-même" docs/manual/fr/*.tex` ne rend plus rien (les deux
   derniers textes provisoires retirés par les 15-1a2-* ou par cette story — validation P3, R L-7) — le PDF aplati ne la porte pas, `pdftotext` coupant les mots
   à la césure (« d’ellemême » pour « d'elle-même » sur le PDF actuel : un « vien-dra » passerait). Les **PDF**
   régénérés (`make fr` dans `docs/manual/` — les trois) et aplatis (`pdftotext docs/manual/fr/user-manual.pdf - |
   tr '\n' ' ' | tr -s ' '`) prouvent la régénération et la présence du titre de la section neuve ; la brochure
   aplatie ne range plus le lettrage au backlog.
4. Tout numéro de compte cité en exemple existe dans un plan livré, sous son nom (G4-bis) ; DE/IT/EN : les manuels
   n'y sont que des `README.md` — rien à traduire.

**AC13 (part ii) — E2E** (`frontend/tests/e2e/open-items.spec.ts`, complété ; mêmes montage et garde #326 que la
15-1c-i) : (7) après le lettrage du scénario (2), la **fiche d'écriture** montre le code dans la colonne
« Lettrage », et le lien ouvre le groupe (`?group=`) ; (8) au **Grand livre** du compte créé, la colonne porte le
code et le lien « Postes ouverts de ce compte » ouvre `/open-items` sur ce compte, à la fin de la période ;
(9) la fiche d'une écriture lettrée montre le motif `ENTRY_LETTERED` **avec** le lien vers le groupe. ⚠️ Le
scénario (3) de la 15-1c-i délettre le groupe du scénario (2) (validation P3, R L-4) : les scénarios (7) à (9)
**posent leur propre lettrage** (deux écritures opposées à montants uniques sur le compte créé, lettrées par
l'API `POST /letterings`) et le délettrent à la fin — ils ne dépendent d'aucun autre scénario. **Lancée au dernier commit de code** (D7), suite complète, jugée
contre `docs/testing.md` § « Les échecs attendus ».

**AC14 — CHANGELOG** (`[0.13.0]`) : **une** entrée cohérente pour le lettrage sous *Ajouté*, non des fragments.

- Inventaire au T0 : `grep -n '518' CHANGELOG.md` — chaque entrée relue.
- L'entrée de la 15-1a-i (« **Le lettrage manuel, par l'API** (#518) ») est **réécrite** : son **titre** cesse de
  dire « par l'API », la phrase « **L'écran viendra** : dans cette version, le lettrage manuel se fait par l'API… »
  est retirée, et l'entrée dit l'écran *Mensuel → Postes ouverts* (compte et date, motifs, lettrage manuel et
  propositions, délettrage), la colonne de la fiche d'écriture et du Grand livre. Les ajouts des 15-1a2 et de la
  15-1b (pièces lettrées d'office, routes des postes ouverts et des propositions) y sont **fondus** si elles les ont
  écrits en entrées séparées sous *Ajouté*.
- Sous *Modifié*, les changements de contrat **additifs** restent des entrées propres (patron `CHANGELOG.md`
  « ⚠️ Changement de contrat… »), écrites par les stories qui les font : celui de la 15-1b (`letterable`,
  `letteringCode` du Grand livre) et celui de la **15-1c-0** (`GET /api/v1/letterings/{key}` enrichi, 15-1c-0
  AC18 ; C-15-1c-21, qui révise C-15-1c-10). Cette story les **relit** — titres et renvois cohérents avec l'entrée
  *Ajouté* — sans les réécrire.
- Toute phrase « à ce stade, Kesh ne … pas encore » encore présente est relue contre l'état livré.

**AC17 — Les autres supports.**

- `docs/api-external.md`, section « Lettrer des lignes » : « *(Depuis la v0.13.0 ; l'écran viendra.)* » devient
  « *(Depuis la v0.13.0 ; à l'écran : Mensuel → Postes ouverts.)* » ; inventaire `grep -n -i "viendra\|pas encore"
  docs/api-external.md`, chaque site classé (la mention « aucun groupe `document` n'existe encore » appartient à la
  15-1a2-i, C-15-1a2-24 : vérifier qu'elle l'a retirée, sans la réécrire ici).
- `README.md` : section *Fonctionnalités* — une ligne pour le lettrage et les postes ouverts ; *Feuille de route*,
  ligne v0.13.0 — la 15-1c-ii passe à « Livré » (chaque story précédente l'a fait dans sa PR : 15-1c-0 AC18,
  15-1c-i AC19), et la ligne ne nomme plus le lettrage parmi « À venir ».
- `docs/manual/fr/marketing-brochure.tex` (validation P2, R M-5 = F2-5 ; C-15-1c-19) : le lettrage sort du bloc
  « Backlog (Epic 13 à 15) » pour un bloc ou une ligne de ce qui est livré — les postes ouverts, le lettrage manuel
  avec propositions, les pièces lettrées d'office —, **sans** promettre « justificatifs » ni « journaux
  personnalisables », qui restent au backlog tant qu'ils ne sont pas livrés. PDF régénéré (AC12 point 3).
- `website/roadmap.html`, bloc **E15** (« automated payment matching to invoices ») : réécrit pour dire ce qui est
  livré — les postes ouverts d'un compte à une date, le lettrage manuel avec propositions, les pièces lettrées
  d'office — sans promettre un appariement automatique que Kesh ne fait pas (il **propose**, `CLAUDE.md`) ;
  `website/index.html` : si une liste de fonctionnalités y figure, la même ligne. Aucune promesse au-delà de `main`.

## Tasks

- [ ] **T0** — Rebaser sur `main` après le merge de la 15-1c-i ; inventaires d'AC12 (manuels), d'AC14
      (CHANGELOG), d'AC17 (`api-external.md`) ; bornes de `i18n-keys.test.ts` ; écrire au Dev Agent Record.
- [ ] **T1** (AC9) — fiche d'écriture : colonne, lien du motif `ENTRY_LETTERED`.
- [ ] **T2** (AC9) — Grand livre : type `LedgerLine.letteringCode`, colonne, `colspan` calculés, lien « Postes
      ouverts de ce compte » (comptes lettrables passés par la page des rapports).
- [ ] **T3** (AC11 part ii) — clés ×4 locales, gardes.
- [ ] **T4** (AC12) — manuel FR, propagation par inventaire sur `docs/manual/fr/*.tex`, preuve négative sur les
      sources, PDF régénérés et contrôlés aplatis.
- [ ] **T5** (AC14, AC17) — CHANGELOG (*Ajouté* fondu, *Modifié* relus), `api-external.md`, README, site, brochure
      (`marketing-brochure.tex` et son PDF).
- [ ] **T6** — Tests (liste ci-dessous) ; E2E (AC13 part ii).

**Tests de T6** — un par ligne, rattaché à son critère :

1. AC9 — fiche d'écriture (Vitest de la page ou de son composant de ligne) : code en lien vers `?group=` sur une
   ligne lettrée, cellule vide sur une ligne ouverte, en-tête traduit ; somme des `colspan` de chaque `<tr>`, pied
   *Total* compris, égale au nombre d'en-têtes, **avec et sans** projets.
2. AC9 — motif `ENTRY_LETTERED` : le code du motif est un lien ; les autres motifs (pièce, exercice) restent du
   texte ; Consultation ne voit pas de motif (C-15-8-14, inchangé).
3. AC9 — `GeneralLedgerView` : colonne du code, **la dernière** ; pour **chaque** `<tr>` du corps et du pied, **la
   somme des `colspan`** (1 par cellule sans attribut) **égale le nombre d'en-têtes** — ouverture, rupture, clôture,
   ligne vide, ligne de mouvement **et total des mouvements** (validation P2, F2-6 : comparer chaque `colspan` au
   nombre de colonnes échouerait sur le code juste, 7 + 1 ≠ 8) ; dans la ligne du total, les cellules du débit et
   du crédit sont à l'**index** des en-têtes « Débit » et « Crédit » (`GeneralLedgerView.test.ts` lit aujourd'hui le
   texte des cellules, aucun `colspan`) ; lien présent **ssi** le compte est lettrable, avec `asOf` = fin de
   période ; aucun lien si la liste des comptes est vide.
4. AC11 — gardes i18n et bornes (tests existants, bornes relevées).
5. AC12 — contrôles documentaires exécutés et leur sortie notée au Dev Agent Record : les deux `grep` d'inventaire
   sur `docs/manual/fr/*.tex` (avant, après), le `grep` négatif d'AC12 point 3 (vide après), le `pdftotext` aplati
   des trois PDF (titre neuf présent, brochure sans lettrage au backlog).
6. AC14, AC17 — contrôles documentaires (validation P2, R L-10), sortie au Dev Agent Record :
   `grep -n -i "viendra\|par l.API\|pas encore" CHANGELOG.md docs/api-external.md` — chaque site restant classé ;
   `grep -c '518' CHANGELOG.md` avant et après (les entrées *Ajouté* fondues, les *Modifié* gardées) ; relecture du
   bloc E15 de `website/roadmap.html`, de `website/index.html` et de la ligne v0.13.0 du README.

*E2E* : AC13 part ii, scénarios (7) à (9).

*Tests existants à relire* : `GeneralLedgerView.test.ts`, `blocker-messages.test.ts`, `form-helpers.test.ts`
(`ENTRY_LETTERED` → `stale`, inchangé), les tests de la fiche d'écriture ; `reports.spec.ts` (colonnes du Grand
livre, s'il les compte).

## Dev Notes

- **Modules — aux deux grains** (C-15-1a2-21) : crates et paquets — `frontend`, `kesh-i18n` = **2** ; modules de
  premier niveau — `features/reports` (Grand livre) et `routes/(app)/reports` (la liste des comptes passée à la
  vue), `routes/(app)/journal-entries/[id]` (colonne, motif) — **trois de logique** — et l'E2E, puis la propagation
  de textes : `kesh-i18n` (catalogues), `docs/manual/fr` (+ PDF), `docs/api-external.md`, `CHANGELOG.md`, `README.md`,
  `website/`, `shared/i18n-keys.test.ts` — **sept** : **11 au grain fin**. Le dépassement ne vient que de la
  propagation de textes : dérogation sur le patron exact de C-15-1a2-23 (C-15-1c-11), plus bas.
- Aucune migration, aucun code serveur. Gate backend complet quand même au dernier commit de code (FTL de
  `kesh-i18n`).
- `\keshVersion` et les macros de version des manuels relèvent de la **release** (`CLAUDE.md`, point 4-bis), pas de
  cette story.
- **Après merge** : vérifier `gh issue view 518 --json state` (le mot-clé `closes #518` est sur la PR, squash).

## Dérogation règle de splitting

Au grain des crates et paquets, la story est à 2, sous le seuil. Au grain fin, 11 : **trois** modules de logique —
la fiche d'écriture, le Grand livre et sa page — et l'E2E, puis sept supports de **texte** (catalogues ×4, manuel et
PDF, `api-external.md`, CHANGELOG, README, site, bornes du test de comptage), qui ne portent aucune règle. Décision
de l'orchestrateur, **C-15-1c-11**, sur le patron de **C-15-1a2-23** : pas de découpage. L'alternative — une story
de documentation seule — séparerait le manuel du code qu'il décrit (règle d'inclusion du `CLAUDE.md`). Accepted
risk : une passe de revue relit la propagation des textes, **et le PDF**, comme un axe à part entière (`CLAUDE.md`,
« Le prompt d'une passe doit NOMMER le manuel »).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Création — 2026-10-09 (Opus 5.5, remédiation de la validation P1 de la 15-1c, en autonomie)

Fiche créée par le découpage de la 15-1c (registre C-15-1c-1 ; R-10 = F-10). Bilan par finding de la validation
P1 : voir le Change Log de l'index `15-1c-proposition-ecran.md`. Recompté depuis ce fichier
(`grep -c '^\*\*AC[0-9]'`, `grep -c '^- \[ \] \*\*T'`, `grep -cE '^[0-9]+\. AC'`) : **6 critères** (AC9, AC11
part ii, AC12, AC13 part ii, AC14, AC17), **7 tâches** (T0–T6), **5 tests** numérotés et 3 scénarios E2E.

### Validation P2 — 2026-10-09 (Opus 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, en autonomie)

Rapports : `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p2-{R,F}.md`. Remédiés ici : R M-2 = F2-2 (le sens,
au lien du Grand livre et au manuel — C-15-1c-16), R M-4 (le délettrage d'un groupe de pièce : « il suit la pièce
et ses règlements », jamais « annulez le règlement » — C-15-1c-18), R M-5 = F2-5 (la brochure dans l'inventaire et
à AC17 — C-15-1c-19), F2-6 (colonne « Lettrage » en dernier, test par somme des `colspan`, total du pied compris —
C-15-1c-20), R L-5 = F2-L7 (preuve négative sur les `.tex`, deux apostrophes — C-15-1c-23), R L-6 et F2-L8 (README
et CHANGELOG *Modifié* tenus par chaque story dans sa PR — C-15-1c-21), R L-10 (test 6). La 15-1c-0 est extraite de
la 15-1c-i (C-15-1c-14) : l'ordre gagne un maillon. Bilan complet : Change Log de l'index. Recompté depuis ce
fichier (`grep -c '^\*\*AC[0-9]'`, `grep -c '^- \[ \] \*\*T'`, `grep -cE '^[0-9]+\. AC'`) : **6 critères**
(inchangé), **7 tâches** (inchangé), **6 tests** numérotés et 3 scénarios E2E.

### Validation P3 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, en autonomie)

Prompt `15-1c-validate-prompt-p3.md` ; rapports `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p3-R.md` et `-F.md`.
**R : 0 MEDIUM / 10 LOW ; F : 0 MEDIUM / 9 LOW** — aucun MEDIUM+ ; les « 0 » vérifiés par l'orchestrateur (axes
déclarés exercés par les deux lentilles, recoupés au code : séquence de `dissolve_group_in_tx`, statuts des refus,
`colspan`, sites du manuel). LOW appliqués ici : R L-5 = F-1 (pied *Total* de la fiche d'écriture), R L-4 (scénarios 7 à 9 indépendants), R L-6 = F-9 (30 sites, dont 5 du manuel d'administration ; « trois de logique et l'E2E »), R L-7 (preuve négative élargie), F-7 (garde `i18n-entrees-a-variables`). Bilan complet : Change Log de l'index. Recompté : **6** critères, **7** tâches, **6** tests.
