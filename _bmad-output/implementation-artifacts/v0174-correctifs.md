# Story v0174-correctifs : la 0.17.4 — les bugs de l'usage réel (#819, #888, #889, #869, #872, #817, #842, #540, #901) — fiche-index

Status: ready-for-dev

*Fiche-index rédigée le 2026-11-05, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0174c`, branche
`story/v0174-correctifs` partie de `7bd4d5c0` (le merge de la rétrospective de la série 0.17 sur `main`). Release **0.17.4**,
**sans migration**. Liste acceptée par le Project Lead le 2026-11-05 ; **but de la release : moins de bugs**. Registre :
`v0174-choix-autonomes-v0174c.md`, choix `C-v0174c-1` à `C-v0174c-21`. Clé courte `v0174c` (port 3179, bases `kesh_v0174c` /
`kesh_e2e_v0174c`). Issues en `refs` seulement dans les commits ; la fermeture se porte sur la PR de release. D'autres
stories de la 0.17.4 (dont `v0174-session`) travaillent en parallèle sur d'autres branches : CHANGELOG,
registre et `sprint-status.yaml` se fusionnent **par union** à l'intégration. Numéros de ligne de `7bd4d5c0`.*

## Découpage — trois fiches sœurs (règle de splitting préventif, C-v0174c-1)

Le périmètre touche plus de cinq modules. Il se développe en **trois fiches sœurs**, sur la même branche, chacune close par ses
commits, son gate (complet, E2E complet au dernier commit de code) et sa revue de code avant la suivante :

| Fiche | Issues | Modules | Nature |
|---|---|---|---|
| **`v0174-correctifs-1.md`** — l'import CAMT | #819 (P2) | `kesh-import` (`src/camt053/mod.rs`, `lib.rs`, `tests/camt053_{tests,golden}.rs`, `tests/fixtures/camt053_golden/v04_characterization.0.17.4.json`, `tests/fixtures/README.md`) ; `kesh-core` (`src/bank_imports.rs`) ; `kesh-api` (`src/routes/bank_imports.rs`, `tests/bank_imports_e2e.rs`) | lecture du XML, clé de doublon |
| **`v0174-correctifs-2.md`** — le serveur | #869 (P3, serveur), #817 (P2, serveur), #842 (P3) et #540 (P2), #901 (P4) | `kesh-api` (`src/errors.rs`, `src/address_input.rs`, `src/main.rs`, tests `period_lock_e2e.rs`, contacts, `onboarding_e2e.rs`, verrou) ; `kesh-qrbill` (`src/validation.rs`, `src/lib.rs`) ; `kesh-db` (`src/pool.rs`, `tests/pool_session_reset.rs`) ; `kesh-reconciliation` (`src/mutex.rs`, `src/errors.rs`, commentaires) ; `docs/api-external.md` — quatre modules | refus, pool |
| **`v0174-correctifs-3.md`** — les écrans | #888 (P3), #889 (P4), #872 (P3), #869 (P3, écran), #817 (P2, écran) | `frontend` (`payment-batches`, `supplier-invoices`, `contacts`, et le *rollout* de #889 : 13 fichiers de dates du jour et 6 d'affichages, dont trois communs — **16 fichiers distincts**, 17 avec `users` —, la garde) ; `kesh-i18n` (trois clés, quatre locales) | écrans ; #889 mécanique, revu fichier par fichier |

**Ordre** : la fiche 1 et la fiche 2 dans l'ordre qu'on veut (aucun fichier commun hors CHANGELOG, manuels, registre) ; **la
fiche 3 après la fiche 2** (#869 : l'écran lit `details.lockedThrough` ; #817 : `addressCountryKnown`). Clés de statut : `v0174-correctifs` (cette
fiche, qui passe `done` quand les trois le sont), `v0174-correctifs-1`, `-2`, `-3`.

## Contrainte — production et migrations

- **Kesh 0.17.3 tient des livres réels sur le NAS du Project Lead** — relevés bancaires importés, factures fournisseur,
  écritures. Aucun essai sur le NAS, sur `~/kesh/`, `~/kesh-test/` ni sur une donnée réelle ; fixtures synthétiques, bases
  `#[sqlx::test]` jetables, base de démonstration jetable `kesh_v0174c` pour les captures. Dépôt public : aucune donnée réelle
  dans un test, une fiche ou un commit.
- **Aucune migration** (C-v0174c-2) : `ls crates/kesh-db/migrations/*.sql | wc -l` vaut **89** avant et après (recompté à
  `7bd4d5c0`). Aucune donnée existante n'est réécrite : les transactions CAMT importées gardent leur texte tronqué (#819), les
  adresses au pays invalide le gardent (#817). `kesh_version_min_required` inchangé ; **cette story** laisse le retour à la 0.17.3
  possible sans restauration ; celui de la release dépend aussi de `v0174-session`, seule susceptible de porter une migration.
- **Hors périmètre** (release, faits par l'orchestrateur) : bump Cargo, macros des manuels (`kesh-style.sty`), README, tag. Les
  PDF des manuels sont régénérés par le développeur pour les seuls textes modifiés ici.

## Gates (chaque fiche)

Tests rouges d'abord (constatés et notés), correctif, mutations nommées (sous le verrou de gate, restauration
`git checkout -- <fichier>` puis `touch`), gates, captures, commit `refs #N`, revue de code. Toute commande lourde passe par
**les deux verrous**, une commande par prise, `free -g` avant (sous 12 Go disponibles, attendre) :

```sh
flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh <commande>
```

- Backend : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/test-fast.sh --ci`,
  `cargo build --workspace --all-targets`, `cargo publish --dry-run -p kesh-import` (fiche 1 : le crate publiable change) ;
  base remise à neuf avant chaque gate complet (bases `kesh_v0174c`, `kesh_e2e_v0174c` recréées par les migrations du
  worktree et `scripts/seed-dev-db.sql` ; **jamais** de redémarrage du conteneur). Fiche 2 : **gate complet même en cours de
  boucle** (`kesh-db`, exception du `CLAUDE.md`).
- Frontend : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`.
- **E2E complet au dernier commit de code de chaque fiche** (port 3179, `/tmp/kesh-e2e-v0174c-r<n>/…` neufs,
  `KESH_COOKIE_SECURE=false`, recette de `docs/testing.md` ; backend lancé dans la prise de verrou, descripteurs fermés
  `8>&- 9>&-`, tué par son PID ; `ss -ltnp` avant et après) : 7 à 10 échecs attendus, jugés fichier par fichier contre
  `docs/testing.md` § *Les échecs attendus*.
- Le Dev Agent Record n'affirme que ce qui a tourné.

## Manuel

PDF régénéré par `make fr` dans `docs/manual/` (sous les deux verrous), contrôlé **aplati** sous une locale UTF-8 —
`LC_ALL=C.UTF-8 pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/’/'/g" | sed -E 's/([[:alpha:]])- ([[:alpha:]])/\1\2/g'`
— puis `grep -F` de chaque phrase neuve **et** de chaque phrase retirée ; seuls les PDF dont le texte change sont commités.

**`docs/manual/fr/user-manual.tex`**
- **#819** (fiche 1) — § *Détection des doublons* (`:2693-2708`) : depuis la 0.17.4, un nom, un libellé ou une référence qui
  contient `&`, `<`, une lettre codée ou une section CDATA est lu **en entier** ; les transactions déjà importées **ne sont pas
  relues** (Kesh ne garde pas le fichier) et gardent leur texte d'origine ; un fichier déjà importé reste signalé comme tel,
  et une ligne déjà importée, reprise par un autre fichier, reste reconnue comme doublon. § des règles d'affectation
  (`:2886`) : une règle *Contrepartie exacte* écrite sur un nom lu tronqué par une version antérieure ne reconnaît plus le nom
  entier — la corriger. Un fichier dont une entité est inconnue est refusé : la phrase entre au § *Import dans Kesh*, après
  celle de la devise non supportée (`:2410`), avec le message que l'écran affiche alors (`BankImportParseFailed`, genre
  `MALFORMED_XML`, `routes/bank_imports.rs:884-888` — texte à relever au code et à citer tel quel) ; le manuel n'a aucune liste
  des refus de fichier à ce jour.
- **#817** (fiche 2) — `:213` (assistant) et `:947` (adresse d'un contact) : le pays est un code ISO 3166-1 alpha-2
  **connu** (« CH », « FR », « DE »…) ; un code inconnu (« SU » pour « Suisse », « XX ») est refusé avec un message qui le
  nomme ; un contact enregistré avant la 0.17.4 avec un tel code se lit toujours, **se modifie** sans toucher au pays, et
  l'écran signale le code inconnu (la QR-facture de ce contact ne pourra pas être produite) ; un pays changé doit être valide.
- **#817** (fiche 3) — la même phrase de `:947` dit l'avis de l'écran.
- **#869** (fiches 2 et 3) — `:1792-1793` (repassage en brouillon d'une facture fournisseur, verrou de période) : le refus
  **nomme la date du verrou**.
- **#872** (fiche 3) — `:1696-1700` (*Modifier* un brouillon fournisseur) : un projet archivé déjà imputé reste affiché,
  marqué « (archivé) » ; la validation le refuse toujours (`:1705`) — choisissez-en un autre avant de valider.
- **#888** (fiche 3) — `:1996-1998` (*Paiements par fichier pain.001*) : **toutes** les factures validées sont proposées,
  quel que soit leur nombre ; un lot ne dépasse pas 500 factures (le refus le dit).
- **#889, #842, #540, #901** : rien à décrire (`grep -n -i 'date du jour' docs/manual/fr/user-manual.tex` à joindre : aucune
  phrase ne décrit l'heure de bascule ; la borne du verrou, `:641`, reste juste — le serveur juge à sa date, C-v0174c-14).

**`docs/manual/fr/admin-manual.tex`** — une section *Passer à la 0.17.4* est de la release (orchestrateur) ; cette story lui
fournit, au rapport, les points à y porter : #819 (rien n'est relu, les règles « nom exact »), #817 (les contacts et la
société au pays invalide — et la société **ne peut pas** corriger son adresse par l'écran, #909), #842 (aucune
action). `docs/manual/{de,en,it}/` : `README.md` seulement, rien à traduire.

**`docs/api-external.md`** (fiche 2) — le début de `:493` (« quatre autres », « 7 avec et 4 sans » ; la fin, « trois cas sur
cinq », reste vraie), des incises à `:298` et `:336`, une ligne `PERIOD_LOCKED` au § 10 (`:937-`).

## CHANGELOG

Section commune `## [0.17.4] — Non publié`, **au-dessus** de `## [0.17.3] — 2026-11-05` (`CHANGELOG.md:11`) ; les deux autres
stories y écrivent aussi : **union**, sans réécrire leurs puces ; l'intro (« version corrective… ») est celle de la release et
se vérifie à l'intégration contre `v0174-session`.

`### Corrigé`
- **L'import CAMT.053 lit les noms et libellés qui contiennent « & », une lettre codée ou une section CDATA**
  ([#819](https://github.com/guycorbaz/kesh/issues/819)) : « Dupont & Fils » était lu « Fils », un libellé en CDATA était vide.
  Les transactions déjà importées ne sont pas relues ; un fichier ou une ligne déjà importés restent reconnus comme doublons.
  Une règle d'affectation *Contrepartie exacte* écrite sur un nom tronqué est à corriger. Un fichier qui porte une entité
  inconnue est désormais refusé.
- **Le pays d'une adresse est contrôlé sur la liste ISO** ([#817](https://github.com/guycorbaz/kesh/issues/817)) : « SU » ou
  « XX » sont refusés à la saisie d'un contact ou dans l'assistant, au lieu de faire échouer la QR-facture plus tard. Un
  contact déjà enregistré avec un tel code se lit et se modifie toujours ; l'écran signale son pays inconnu, et un pays
  changé doit être valide.
- **Toutes les factures validées sont proposées pour un lot de paiement** ([#888](https://github.com/guycorbaz/kesh/issues/888)),
  non plus les 200 premières.
- **La date du jour est celle de votre poste, non plus celle d'UTC** ([#889](https://github.com/guycorbaz/kesh/issues/889)) :
  entre minuit et 1 h (2 h en été), la facture, le règlement, le solde, le rappel manuel, l'avoir, le lot de paiement, le
  paiement d'une facture fournisseur, l'expiration minimale d'une clé d'API et les noms des fichiers exportés proposaient la
  veille. L'envoi d'un document, le gel d'un PDF, un rappel, la confirmation d'un lot, la date d'import d'un document et la
  création d'un utilisateur s'affichent à leur date suisse.
- **Le refus de repasser une facture fournisseur en brouillon sous le verrou de période dit jusqu'à quand**
  ([#869](https://github.com/guycorbaz/kesh/issues/869)). Pour l'API : `400 PERIOD_LOCKED` porte `details.lockedThrough` et
  `details.attempted` (`docs/api-external.md`).
- **Un brouillon de facture fournisseur montre son projet archivé** ([#872](https://github.com/guycorbaz/kesh/issues/872)),
  marqué « (archivé) », même quand son projet parent l'est aussi.
- **Une requête interrompue ne bloque plus un compte bancaire** ([#842](https://github.com/guycorbaz/kesh/issues/842)) : le
  verrou du compte, et l'état des clés étrangères d'une restauration interrompue
  ([#540](https://github.com/guycorbaz/kesh/issues/540)), sont rendus au retour de la connexion au pool, au lieu d'y rester
  jusqu'à une demi-heure.
- La 0.17.1 est datée du 2026-11-04, jour de sa publication ([#901](https://github.com/guycorbaz/kesh/issues/901)).

`refs #819`, `refs #888`, `refs #889`, `refs #869`, `refs #872`, `refs #817`, `refs #842`, `refs #540`, `refs #901` aux commits ;
**aucun mot-clé de fermeture** (contrôle :
`git log origin/main..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` vide ; écrire
« validation terminée », jamais « close »).

## Dev Notes communes

- **Jamais de donnée réelle** : XML CAMT, contacts, factures et projets inventés.
- **On trace toujours** — défauts voisins relevés à la fiche, non traités. **Ouverts** par l'orchestrateur : **#906** la liste
  des lots de paiement ne lit que les cent premiers (C-v0174c-12) ; **#907** la borne du verrou de période est jugée à la date
  UTC du serveur (`crates/kesh-db/src/repositories/companies.rs:1013`, `:1100`), si bien qu'entre minuit et 1 h (2 h en été)
  verrouiller jusqu'à la veille est refusé (C-v0174c-14) ; **#908** les messages du serveur de `PERIOD_LOCKED` (et
  `journal-entries-period-locked`) portent des dates ISO (C-v0174c-8) ; **#909** aucun écran ne corrige l'adresse de la
  société après l'assistant — une société enregistrée avec un pays invalide ne peut plus produire de QR-facture (C-v0174c-9) ;
  **#910** un sélecteur de pays commun aux écrans d'adresse (C-v0174c-9). **Demandés** au rapport : (6) `prepare-release.sh`
  date le CHANGELOG à la préparation, non au tag (C-v0174c-20) ; (7) aucune borne haute à la lecture des factures d'un lot —
  10 000 factures validées non payées feraient 50 requêtes (C-v0174c-12) ; (8) « et l'exercice » de #869, non retenu
  (C-v0174c-8), si le Project Lead le veut ; (9) les angles morts de la fiche 2 : le chemin `RELEASE_LOCK` raté sans montage
  (AC-2.12) et l'application du crochet par `main.rs`, qu'aucun test ne voit (MB-842-b). Les limites déclarées sans issue, et
  pourquoi : le faux doublon possible par alias (C-v0174c-5 — signalé à l'aperçu, borné aux lignes dont la clé change) et la
  facture manquante d'une lecture par pages pendant une validation concurrente (fiche 3, § 3.1 — la relecture la retrouve).
- **#540** (P2) est intégrée à la fiche 2 (C-v0174c-11) : `refs #540` ; `closes #540` proposé à la PR si la revue le confirme.

## Change Log

- 2026-11-05 — fiche-index et trois fiches sœurs créées (Opus 5.5, autonomie). Prémisses établies au code (`7bd4d5c0`) :
  #819 confirmée, clé du relevé indépendante de la lecture, clé de ligne dépendante → alias hérité (fiche 1) ; #869, #817,
  #842, #901 confirmées (fiche 2), #540 intégrée ; #888 confirmée, #889 confirmée (12 sites corrigés sur 15, 9 affichages sur
  12 — les trois `paidAt` sont des dates, prémisse du commentaire corrigée), #872 corrigée (projet d'en-tête, non de ligne)
  (fiche 3). Registre C-v0174c-1 à 20. Aucune migration. Prochaine : validation P1 (Sonnet, lentilles R et F,
  `v0174-correctifs-validate-prompt-p1.md`).
- 2026-11-05 — **Validation P1 (Sonnet, lentilles R et F) : R 0/1/2/8, F 0/0/4/7** (rapports
  `kesh-gate-logs/v0174c-validate-p1-{R,F}.md`). Chaque prémisse relue au code (et, pour R1, dans la source de quick-xml 0.42)
  avant remédiation ; **aucun finding rejeté**. Sorts : **R1** (HIGH) → C-v0174c-21, fiche 1 ; **R2** (MEDIUM) → pays inchangé
  toléré, `addressCountryKnown`, avis à l'écran (fiche 2 § 2.2, fiche 3 § 3.5, C-v0174c-9 révisé, arbitrage de
  l'orchestrateur) ; **R3 = F4** (MEDIUM) → garde par occurrence, commentaires masqués, motif parenthésé, sonde dans un
  fichier autorisé (fiche 3, C-v0174c-17) ; **F1** (MEDIUM) → la fin de `api-external.md:493` reste vraie ; **F2** (MEDIUM) →
  `:298`, `:336`, § 10 ; **F3** (MEDIUM) → trois sites de plus du symptôme « fin de session » (fiche 2, tâche 4). LOW : R4, R9,
  R11 (fiche 3) ; R5 (squash), R10 = F9 (modules : l'écran de #817 passe à la fiche 3) (fiche 2) ; R6, R7 (fiche 1) ; R8 = F10
  (issues demandées closes, ci-dessus) ; F5 (ancre du manuel) ; F6 (#540 P2, titres) ; F7 (puce #889 : « date de votre
  poste », deux sites ajoutés) ; F8 (lignes) ; F11 (`CLES_RELEVEES` 399 → **402**, et non 401 : la fiche 3 ajoute une troisième
  clé, `contacts-country-unknown`). Issues (1) à (5) du premier rapport : #906 à #910. Grep des valeurs corrigées sur l'index,
  les fiches et le registre : `trim_text`, `addressCountryKnown`, `#906`…`#910`, `:640`, `:64-`, `:132-136`, `401`, `402`,
  `#540 (P3`, `trois cas sur cinq`, `fin de session`, `celle de Zurich` — résidus traités, les entrées datées des Change Logs
  laissées telles quelles. Prochaine : validation P2 (Opus, lentille complète, `v0174-correctifs-validate-prompt-p2.md`).
