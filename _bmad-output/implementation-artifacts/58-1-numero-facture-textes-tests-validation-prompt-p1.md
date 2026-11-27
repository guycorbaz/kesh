# Prompt — validation P1 de la fiche `58-1-numero-facture-textes-tests` (#1115, #1117, #1118, #1119, #1120, #1122, #1123, #1124), passe complète

Tu valides, **en lecture seule**, la fiche de story `58-1-numero-facture-textes-tests` de Kesh (comptabilité suisse ; Rust/Axum, MariaDB,
SvelteKit 5 runes). C'est la première story du premier **epic de nettoyage** (Epic 58, release visée : vendredi 2026-12-04) : huit
issues P3/P4 peu coûteuses du format de numéro de facture, **sans règle métier changée, sans migration, sans changement d'API** (le
Project Lead a dit le 2026-11-26 que changer des phrases de refus, sans changer ni code d'erreur ni champ, n'en est pas un). Kesh tient
les livres réels du Project Lead, qui émet ses factures sous le format par défaut `F-{YEAR}-{SEQ:04}` : aucun numéro, aucune facture,
aucun PDF existant ne doit changer.

- **Modèle : Opus. Contexte frais. Passe complète** (première passe : jamais ciblée, jamais Haiku). Langue du rapport : français.
- Worktree : `/home/gcorbaz/devel/kesh-58-1`, branche `story/58-1-numero-facture-textes-tests`, partie de `epic/58-nettoyage` =
  `ee684a69f`. Fiche :
  `/home/gcorbaz/devel/kesh-58-1/_bmad-output/implementation-artifacts/58-1-numero-facture-textes-tests.md`.
- Lis d'abord : le `CLAUDE.md` du worktree (Qualité du code, Test Locally First, Recette visuelle, Review Iteration Rule, Inventorier
  les sites NON RÉSOLUS, Propagation post-patch, Recompter, Priorités des défauts, Issue Tracking Rule, Règle de splitting préventif,
  Documentation à synchroniser, Migration breaking policy pour constater qu'elle ne s'applique pas) ; `docs/agents-orchestration.md` ;
  `_bmad-output/planning-artifacts/epic-58-nettoyage.md` (section 58-1 et § Règle) ; la fiche de la 35-1
  (`35-1-numerotation-sans-seq.md`) et le registre `e35-choix-autonomes-e351.md` (C-e351-7) ; les huit issues
  (`gh issue view <N> --json number,title,body,comments`, lecture seule, une requête toutes les 10 s) : #1115, #1117, #1118, #1119,
  #1120, #1122, #1123, #1124.

## Interdits

Tu n'écris **aucun fichier** hors de ton rapport. En particulier, tu ne lances **ni** `cargo` (build, test, nextest, clippy, fmt),
**ni** `npm` / `npx`, **ni** `make`, **ni** aucun `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`,
`scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `scripts/recette-montee.sh`, `scripts/install-hooks.sh`), **ni** `docker` /
`docker compose`, **ni** `mariadb` / `sqlx`, **ni** `git commit`, `git checkout`, `git stash`, `git restore`, `git reset`, `git push`,
`git worktree`, **ni** `gh` en écriture (`issue create|edit|comment|close`, `pr …`), **ni** `sed -i`, `tee`, `>` ou `>>` vers un
fichier du dépôt, **ni** `pkill`, `killall`, `kill`. Rien sous `~/kesh/`. Permis : `git log`, `git show`, `git diff`, `git grep`,
`grep`, `sed -n`, `cat`, `pdftotext … -` (vers la sortie standard), `gh … view|list`, et un `python3 -I` qui ne fait que **lire**.
Aucune redirection vers un fichier du dépôt ; ton rapport s'écrit avec l'outil d'écriture, à son seul chemin.

Rapport : **`/home/gcorbaz/devel/kesh-gate-logs/s581/validation-p1.md`** (le seul fichier que tu écris).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Défauts au code.** Chaque défaut est relu à `ee684a69f` avec la ligne citée (`chemin:ligne`, extrait) : `errors.rs:2364-2383`,
   `invoice_format.rs:27`, `:92-95`, `:149-152`, `:214-215`, `:345-347`, `:361`, `:386-387`, `invoice-number-format.ts:14`,
   `:193-203`, `:246-249`, `invoice_pdf_service.rs:479-482`, `kesh-qrbill/src/generator.rs:73`, `invoices.rs:2477-2503`. Une ligne
   fausse, une prémisse démentie (la fiche dit qu'**aucune** des huit n'est déjà corrigée) est un finding. Un défaut que la fiche
   garde alors qu'il est corrigé, ou une issue que la fiche garde alors qu'elle révèle une règle métier, est un finding.
2. **Périmètre : ni règle métier, ni migration, ni API.** Vérifie, issue par issue, que le correctif décrit ne change aucune règle :
   en particulier #1119 (D1 : le serveur reste souple — le refus du `+` côté Rust est-il bien une règle neuve ? un format `{SEQ:+4}`
   déjà enregistré deviendrait-il illisible ?) et #1120 (le miroir ne devient-il plus strict que le serveur ? plus laxiste ?
   `\s` → espace et tabulation ; longueur en octets). Les textes de refus changent-ils un code d'erreur, un champ, un `details` ?
   Aucune migration n'est requise (vérifie : `git diff --stat` des fichiers listés, aucun `crates/kesh-db/migrations/`).
3. **Inventaires du symptôme, ensembles clos.** Refais toi-même, en lecture seule, et compare à la fiche : (a) tous les sites de
   « en compte deux », « counts as two », « zählt zwei », « ne conta due » (code, catalogues, `docs/`, manuels, `frontend/`, fiches
   vivantes) — la liste de la fiche (huit textes + CHANGELOG) est-elle close ? les `mcp-sends-*` sont-ils bien un autre sujet ? ;
   (b) tous les sites de `C-e351-7` ; (c) tous les sites qui présentent la borne de 64 octets comme imposée par la QR-facture ;
   (d) tous les sites qui parlent de caractères pour une longueur mesurée en octets dans `invoice_format.rs` et `invoice-number-format.ts`
   (la fiche en laisse un, `TemplateTooLong`, D2 : le choix tient-il ?). Un site manqué, ou une assertion dont les deux côtés sortent
   du même extracteur (vraie par construction), est un finding.
4. **Critères et tests.** Chaque AC est testable et rougit pour la bonne raison ; les tests vert-au-départ (AC2, AC4 Rust, AC8) ont
   leur mutation, tuée par un test nommé ; les mutations M1 à M8 sont uniques, réalisables, et chacune a bien son test tueur (M8 :
   « valider la transaction avant de rendre le refus » est-elle exécutable à `invoices.rs:2489-2503` ? sinon proposer une autre).
   AC2 : le scénario (`2026-01-01 → 2026-06-30`, puis `2026-07-01 → 2027-06-30`) passe-t-il le contrôle de chevauchement et la
   création d'un exercice par la route ou par SQL ? le test fixe-t-il bien `F-2026-0001` deux fois ? AC7 : un nom d'exercice de
   22 « – » posé par SQL, sous `{FY}-{SEQ:04}`, atteint-il 66 + 5 octets ? AC5 : `'é'.repeat(33)+'{SEQ}'` donne-t-il 71 octets ?
   Sondes positives avant toute assertion d'absence ; aucun test vrai par construction.
5. **Tests qui assertent le texte.** Les tests existants (`textes_coherents.rs` G9, `parity_between_locales`, tests de
   `number_format_unrenderable_e2e.rs`, tests vitest de `invoice-number-format`) pinnent-ils une phrase que la fiche change ?
   Lesquels rougissent, lesquels la fiche oublie ? Le texte nouveau (D4) tient-il aux quatre locales, passe-t-il G9 (repli Rust =
   fr-CH) et reste-t-il sans nouvelle clé (`sitesTotal`, `lint-i18n-ownership`) ?
6. **Cohérence écran / serveur.** Le miroir jugé après correction accepte-t-il exactement ce que `is_allowed_char` accepte ? L'ordre
   des refus (longueur avant caractères, au serveur) est-il reproduit ? `trim().length === 0` du miroir contre `trim().is_empty()`
   (U+FEFF) : écart de motif seulement ? Le « actuel : n » en octets dit-il « caractères » : acceptable (D2) ?
7. **Manuel, PDF compris.** Vérifie `docs/manual/fr/user-manual.tex:1602` **et** le PDF publié, **aplati avant de greper** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o '.\{0,200\}<motif>.\{0,200\}'` (motifs :
   « 64 octets », « comptent chacune deux », « impossible à produire », « SEQ »). Confirme que `docs/manual/{de,en,it}/` ne portent
   que `README.md` ; que `docs/api-external.md:527` et `:1417` sont les seuls passages de l'API ; que le manuel d'administration
   (`admin-manual.tex`) ne décrit pas ces refus. Un écart manuel / code, avant ou après la story, est un finding.
8. **Recette visuelle.** Chaque texte visible touché a sa capture prévue (refus `too_long`, `duplicate`, page des réglages) ; les
   états touchés sont-ils tous couverts ? Aucun thème sombre. La fiche exige-t-elle les captures hors dépôt (`kesh-gate-logs/`) ?
9. **Règle de splitting et gates.** Recompte les modules au grain du domaine (crate ou domaine du frontend dont le **code de
   production** change) depuis la liste des fichiers : la fiche dit quatre ; au-delà de cinq, finding. Les gates ciblé et complet
   sont-ils ceux de `CLAUDE.md` (fmt, clippy, nextest, `textes_coherents`, `kesh-i18n`, vitest, `check`, `lint-i18n-ownership`,
   `build`, E2E complet, `gate-ci.sh`, base remise à zéro, `mem-guard`, `flock`) ? Aucune affirmation de la fiche n'est-elle
   « tournée » alors que rien n'a tourné ?
10. **Rectificatifs et historique.** Les sites 3 et 4 de #1122 (registre, fiche de la 35-1) reçoivent un rectificatif **daté** sans
    réécrire l'entrée passée ; l'entrée publiée du CHANGELOG (0.17.7) n'est pas réécrite (D3) : la décision tient-elle à la règle des
    fichiers communs et à celle des numéros de release ? Aucun mot-clé de fermeture (`closes`, `fixes`, `resolves` et variantes)
    dans la fiche ni dans les messages de commit (`git log origin/epic/58-nettoyage..HEAD --format=%B`).
11. **Notes de version et documentation.** Une puce, sans nommer d'epic, numéro de release seulement ; « Passer à la 0.21.0 » ; pas de
    CHANGELOG ni de PDF dans la story ; `README.md` « Feuille de route » et `website/` : rien à changer ? (justifie.)
12. **Fichiers communs et coordination.** Conflits possibles avec les autres stories de l'Epic 58 (58-3 : manuel et guide de l'API,
    mêmes `api-external.md` et `user-manual.tex` ; 58-4 : mêmes catalogues `messages.ftl`) et avec les autres epics ouverts
    (49 à 54, release 0.20.0 de vendredi) ; `sprint-status.yaml` (clé de la 58-1 du bloc de l'Epic 58).

## Rapport

Pour chaque finding : identifiant `V1-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), axe, constat **avec la ligne lue citée**
(`chemin:ligne` et l'extrait), conséquence, remède proposé. Pour tout CRITICAL ou HIGH qui affirme l'absence d'un code ou la présence
d'un anti-pattern, la commande `grep -nF` qui le prouve et sa sortie.
En fin de rapport : décompte **CRITICAL / HIGH / MEDIUM / LOW**, recompté depuis tes findings ; la liste des **axes exercés** et des
**axes non exercés** (avec la raison) ; les commandes lancées. Un « 0 finding » sans cette liste ne compte pas comme passe.
