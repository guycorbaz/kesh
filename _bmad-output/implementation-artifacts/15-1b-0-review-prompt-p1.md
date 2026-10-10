# Prompt — revue de code P1, Story 15-1b-0 (la propriété des lignes, par lot)

*Versionné le 2026-10-10. Trois lentilles (Sonnet), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-15-1b-0`, branche `story/15-1b-0-propriete-des-lignes-par-lot`.
**Le diff à revoir** : `git diff f8888750 2e86ab4d` (le commit de développement ; `f8888750` est la tête de la 15-1a2-ii,
base de la story ; `HEAD` ne porte que ce prompt). Fiche : `_bmad-output/implementation-artifacts/15-1b-0-propriete-des-lignes-par-lot.md`
— modèle réel, D1 à D5, AC1 à AC7, Tasks, Tests prévus, Dev Notes, et le **Dev Agent Record**, qui déclare trois choix
(C-15-1b-0-5 à 7, registre `epic-15-choix-autonomes.md`, à côté de C-15-1b-0-1 à 4), six mutations (huit lignes) et des
décomptes. Règles : `CLAUDE.md` du dépôt — § « Exception `kesh-db` », § « Inventorier les sites NON RÉSOLUS », § « Recompter
ses propres comptes rendus ». Issue : #518 (la story ne la ferme pas).

**Contexte** : refonte interne, sans effet de contrat déclaré (AC6). `kesh-db/src/repositories/journal_entries.rs` gagne
`DocumentKind` (trois méthodes publiques, source unique), `DocumentOwner` et `document_owners` (une instruction par
tranche de 500, `UNION ALL` de cinq blocs, `MIN(id)` joint en retour) ; `reversal_blockers` / `reversal_blocker` passent sur
`&mut MySqlConnection` et lisent la propriété par `document_owners` ; `modification_blocker` prend une `Transaction` ;
`letterings::first_document_owner` fait un appel par groupe ; `DocumentRef.document_type` est typé. La route
`GET /journal-entries/{id}` lit ses trois motifs dans une transaction de lecture, gardée par un test lexical. La parité est
prouvée contre l'ancien code **gelé** (`crates/kesh-db/tests/document_owners/reversal_blockers_frozen.rs`) et des valeurs
écrites à la main. Frontend non touché.

## Lentilles

- **B — Blind Hunter : bugs, SQL, transactions**, sans présupposer que la fiche a raison. Le SQL de `document_owners`
  **ligne à ligne** : ordre des marqueurs et des liaisons (`[1, 1, 2, 1, 1]`), typage des colonnes de l'`UNION ALL`
  (`CAST`), bloc fournisseur (une facture dont l'achat est l'écriture X et le règlement l'écriture Y ; une écriture achat
  de S1 et règlement de S2), bloc règlement (jointure `invoices` interne : un règlement sans facture est-il possible ?),
  bloc banque (sans jointure de retour), portée par société (les tables des pièces ne sont pas filtrées par société :
  une pièce d'une société B pointant une écriture de A ?), tranches et `BTreeSet`, tri final. **Équivalence avec
  l'ancienne requête** (`git show f8888750:crates/kesh-db/src/repositories/journal_entries.rs`, fonction
  `reversal_blockers`) sur tout état atteignable — où diffèrent-elles, et l'écart est-il nommé par la fiche ?
  `reversal_blockers` : deux lectures — que se passe-t-il entre elles sur une connexion autocommit (prédicteurs,
  `settlement_cancellation`, `modification_guard` sous verrou) ? La route : `begin`/`rollback`, erreur entre les deux
  (la transaction est-elle relâchée ?), interblocage ou attente nouvelle introduite par une transaction de lecture
  (`REPEATABLE READ`, lectures non verrouillantes) ? `first_document_owner` : même résultat que l'ancienne boucle sur
  tout groupe (ordre des lignes, écritures répétées) ?
- **E — Edge Case Hunter : bords et états.** Lot vide, une écriture, exactement 500, 501, 1000, 1001 ; doublons ;
  identifiants négatifs ou nuls ; écriture d'une autre société dans le lot ; pièce dont le numéro est `NULL` (facture
  brouillon, avoir brouillon, facture fournisseur sans numéro) — étiquette `None` comme avant ? deux pièces d'un même
  type sur une écriture (données héritées) ; règlement d'une facture **sans** numéro (`invoice_number` `None`) ; contre-
  passation d'un règlement fournisseur détaché ; écriture possédée ET contre-passée ET au compte archivé. **Les tests
  prouvent-ils** ce qu'ils nomment (le test aurait-il échoué sans le code ? assertion de montage avant chaque assertion
  négative ?) : rejoue mentalement les six mutations du Dev Agent Record et cherche une **mutation plausible qui resterait
  verte** (ex. : `MIN` → `MAX` ; tri final retiré ; `DOCUMENT_OWNERS_BATCH` changé ; `je.company_id` retiré d'un seul
  bloc ; garde lexicale contournée par une autre graphie).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (AC1 à AC7) satisfait ; chaque test de
  « Tests prévus » existe sous son nom et prouve ce qu'il nomme ; l'oracle gelé est-il **réellement** la fonction de
  `056997b0` (`git show 056997b0:crates/kesh-db/src/repositories/journal_entries.rs`, comparer) ? AC3 : plus de boucle
  dans `first_document_owner`, aucune liste de motifs dans son corps ; AC4 : inventaire fermé des appelants
  (`grep -rn "reversal_blockers\|reversal_blocker(\|modification_blocker" crates --include=*.rs`) ; AC6 : `git diff
  --stat f8888750 2e86ab4d -- docs CHANGELOG.md crates/kesh-i18n frontend` vide — **et le comportement visible a-t-il
  vraiment changé nulle part** (étiquettes, codes, ordre des motifs, `NotFound`) ? AC7 : littéraux de `DocumentRef`.
  **Doc-comments par la valeur** : `git grep -nE "Une seule requête|exécuteur par|LIMIT 1|rangs 3 à 6|hors transaction|connexion du pool|connexion acquise|sept causes"`
  sur `crates/` — chaque site encore vrai ? **Décomptes du Dev Agent Record recomptés** (11 tests neufs ; six appels de
  test adaptés dans cinq fonctions et trois fichiers ; trois sites de `DocumentRef` ; 3296 au gate = 3285 + 11).
  **Manuel** : la story déclare ne rien changer de visible ; vérifie-le par la valeur — `grep -rn "contre-pass" docs/manual/fr/*.tex`
  et le PDF aplati (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
  `/home/gcorbaz/devel/kesh-gate-logs/`) : décrit-il un comportement (motif affiché, étiquette, lettrage refusé d'une ligne de
  pièce) que le code livré ne tient plus ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1b-0-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour tout
finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati des PDF au même
endroit) ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont `scripts/prepare-release.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`,
`sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`, `docker`, `gh` en écriture, aucune requête SQL. Autorisés : lecture,
`grep`, `sed -n`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
