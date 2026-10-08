# Prompt — validation P7 ciblée de la spec, Story 15-5e1

*Versionné le 2026-10-08. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur le seul commit de la remédiation P6.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Objet** : `git show ad75ea32`
(lis ce diff, pas la fiche entière ; ouvre la fiche `_bmad-output/implementation-artifacts/15-5e1-socle-rejeu.md`
seulement autour des hunks). Rapports qu'elle remédie : `target/gate-logs/15-5e1-p6-R.md` et `15-5e1-p6-F.md`.
Choix C74 au registre `epic-15-choix-autonomes.md`.

## Lentille unique — chasseur de régressions de la remédiation

1. **Le témoin du rejeu** (montage commun de l'AC4, module `tests/common/capture_rejeu.rs` prévu) : une couche
   `tracing-subscriber` installée par `tracing::subscriber::set_default` capte-t-elle vraiment le `warn!` émis par
   `kesh_db::retry` pendant un `#[sqlx::test]` ? Vérifie au code : la cible réelle du `warn!` dans
   `crates/kesh-db/src/retry.rs` (`grep -nF "warn!" crates/kesh-db/src/retry.rs`), le nom exact du champ
   `operation` tel que la 15-5e1 le pose, le runtime de `#[sqlx::test]` (sqlx-core 0.8.6, `src/rt/mod.rs`), et le
   cache d'intérêt des callsites de `tracing` (un callsite déjà enregistré sous un autre abonné peut-il être
   ignoré ? `tracing::callsite::rebuild_interest_cache`). Un témoin qui ne capte jamais rendrait tous les tests
   rouges ; un témoin qui capte un autre test rendrait un faux vert.
2. **La mutation du témoin** (tests 2 et 7) : prouve-t-elle ce que la fiche dit ?
3. **Le test 7 réécrit** et **T0** : le cycle décrit (S tenu par le test, X attendu par la route, X demandé par le
   test) est-il cohérent avec la séquence de `company_invoice_settings::update` (lis-la) ? La `version` lue par
   `GET` avant la transaction de test est-elle celle que la route compare ?
4. **Le montage** : `seed_accounting_company` crée-t-il vraiment ce que la fiche affirme (réglages, 4 taux,
   exercice 2020-2030, deux Admin) ? `designate_rounding_account` et `disable_rounding_to_5_centimes` existent-ils
   avec la signature citée (`grep -nF`) ? La référence `routes/invoices.rs:1293-1304` est-elle juste ?
5. **Propagation** : « cinq routes », « tests 2 à 5 et 7 », C74, et la cohérence avec la 15-5e2 (AC2, AC5,
   `:85`, `:464`, `:472`), l'index, la fiche mère et la ligne 15-5e1 de `sprint-status.yaml`.

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute
affirmation de présence ou d'absence ; code relu cité), correction proposée. ⛔ **Liste des axes exercés ET non
exercés** — un « 0 finding » sans elle ne compte pas. Rapport complet dans `target/gate-logs/15-5e1-p7-ciblee.md` ;
dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, lecture des
sources dans `~/.cargo/registry/src/`.
