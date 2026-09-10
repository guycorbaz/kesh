# Prompt de la passe 5 de revue de code — Story 24-5

**Versionné** conformément au § « La passe ciblée » du `CLAUDE.md`.

- **Modèle** : Opus 5, contexte frais
  (P1 : Sonnet 4.6 ×2 + Haiku 4.5 · P2 : Opus 5 · P3 : Sonnet 4.6 · P4 : Haiku 4.5)
- **Cible** : la remédiation de la passe 4 — `f7715037..HEAD`, à lire **aplatie**
- **P4** : 0 finding déclaré, **passe NON retenue comme concluante** (voir ci-dessous)

---

Tu es une lentille de revue de code, en **contexte frais**, sur le dépôt Kesh
(`/home/gcorbaz/devel/kesh` — comptabilité suisse, Rust + Svelte). Tu réponds en français.

CIBLE : la remédiation de la passe 4, deux commits (`6c096968` et `a323b1aa`). **Lis-la aplatie**,
pas commit par commit :

```sh
git diff f7715037..HEAD --stat
git diff f7715037..HEAD
```

⛔ **HYPOTHÈSE DE TRAVAIL, MESURÉE SUR CETTE STORY** : *le défaut que tu cherches vient d'être écrit
par la remédiation que tu relis.* En revue de spec, 11/13 puis 3/4 ; en revue de code, 5/6 à la
passe 2, **6/6 à la passe 3**, et 2/2 pour ce que l'orchestrateur a trouvé lui-même après la
passe 4. **Aucune décision d'origine n'a jamais été prise en défaut.**

## Le contexte particulier de cette passe, et il compte

**La passe 4 a rendu « 0 finding » sans avoir fait le travail.** Elle a déclaré elle-même n'avoir
pas couvert l'axe le plus cher (« examen rapide » de l'énumération des chemins d'écriture) ; elle a
qualifié de « robuste » un point qu'elle n'avait pas exécuté — et il était faux ; et elle a **écrit
dans le dépôt** malgré une interdiction explicite, en lançant `scripts/prepare-release.sh`, qui a
bumpé les dix crates de `0.11.1` à `0.12.0` (restauré depuis).

**C'est l'orchestrateur qui a ensuite trouvé les deux défauts**, en refaisant le travail annoncé.
Ta tâche est donc double : relire la remédiation **et** contrôler que ce que l'orchestrateur affirme
avoir établi l'est réellement. *Il est ici dans la position de l'auteur qui se relit lui-même — la
position exacte que ce protocole existe pour corriger.*

## Ce que la remédiation a produit

1. **Une garde serveur** : `crates/kesh-db/src/repositories/supplier_invoices.rs`, validation du
   compte de charge — `SELECT active, account_type` → `SELECT active, postable, account_type`, et
   le `match` passe à `Some((true, true, ref t)) if t == "Expense"`.
2. **Un test négatif neuf** : `create_with_non_postable_expense_account_is_rejected`
   (`crates/kesh-db/tests/supplier_invoices_repository.rs`).
3. **`scripts/prepare-release.sh`** : l'échec de `cargo run --example` devient **fatal** (`exit 1`)
   au lieu d'être avalé par `|| true` ; `stderr` est capturé dans un fichier `mktemp` séparé.
4. **Le journal de la passe 4** et la ligne de gate au story file, plus `sprint-status.yaml`.

## Tes axes d'attaque, par rendement attendu

- ⛔ **LE TEST NEUF N'A PAS ÉTÉ ÉPROUVÉ PAR MUTATION — et l'orchestrateur le sait.** Les deux tests
  de la passe 3 l'ont été (gardes neutralisées ⇒ ils rougissent) ; **celui-ci ne l'a pas été**.
  Fais-le. Neutralise la garde (`Some((true, _, ref t))`) et vérifie qu'il rougit. Vérifie aussi
  qu'il teste ce qu'il prétend : le compte `4000` de `one_line` est-il bien de type `Expense` et
  bien celui que la ligne utilise ? Le test passerait-il pour une **autre** raison — compte
  inexistant dans le seed, `unwrap_err` satisfait par une erreur sans rapport ? *Un test qui passe
  ne prouve pas qu'il teste.*
- ⛔ **LE SIXIÈME TOUR DU RAPPEL DE RELEASE.** Cet artefact a été pris en défaut **cinq fois** en
  trois passes. Il vient d'être retouché. **Exécute-le**, ne le lis pas seulement : le `mktemp`
  a-t-il un `trap` (que se passe-t-il si le script sort entre sa création et son `rm -f` ?) ; le
  `if ! VAR=$(...)` se comporte-t-il comme attendu sous `set -euo pipefail` ; que rend-il si
  `cargo` réussit mais n'imprime rien ; si `mktemp` échoue ; si le dépôt n'a aucun tag ; si un tag
  est annoté plutôt que léger. `bash -n` **puis** exécution réelle des deux branches.
- ⛔ **L'ÉNUMÉRATION EST-ELLE VRAIMENT CLOSE ?** L'orchestrateur affirme que « seuls les cinq appels
  de la réconciliation restent ouverts côté serveur ». **Refais l'énumération toi-même**, sans te
  fier à la sienne, et par plusieurs chemins : `grep` sur `create_in_tx`, mais aussi sur les
  `INSERT INTO journal_entry_lines` / `journal_entries` directs, les repositories qui écrivent des
  écritures sans passer par `journal_entries`, et les routes d'import ou de soldes de départ.
  ⚠️ *Un cinquième chemin a été trouvé après quatre passes ; rien ne dit qu'il n'y en a pas un
  sixième.*
- ⛔ **LA GARDE NEUVE CASSE-T-ELLE UN CAS LÉGITIME ?** L'orchestrateur affirme que non, au motif que
  les trois plans livrés ne portent que 9000/9100/9200 comme non-postable. **Vérifie-le**, et va
  plus loin que lui : `is_postable` a d'autres causes que le champ JSON (rôle `CurrentYearResult`,
  compte parent). Un compte de charge **parent** peut-il être légitimement visé par une facture
  fournisseur ? Un `PUT /api/v1/accounts/{id}` peut-il rendre non-postable un compte déjà utilisé,
  et que devient alors une facture en cours ?
- **LE MANUEL DIT-IL VRAI, AU CINQUIÈME TOUR ?** Trois passes l'ont pris en défaut dans trois sens.
  Il affirme désormais que la validation de facture, l'enregistrement d'un règlement refusent, et
  que « reste le rapprochement bancaire ». Est-ce **exactement** vrai maintenant ? Le PDF
  correspond-il au `.tex` (⚠️ aplatis le texte : `pdftotext` coupe les lignes) ?
- **LES AFFIRMATIONS DU JOURNAL DE PASSE 4 SONT-ELLES EXACTES ?** Il énonce des faits vérifiables :
  l'horodatage du bump (21:26:10), le fait que `CHANGELOG.md` soit resté intact, « 2299 + 1 = 2300 »,
  « les 3 plans ne portent que trois comptes non-postable », « `journal_entries:2952` est dans
  `mod tests` ». **Recompte et revérifie depuis la source.** Un compte rendu faux est un finding.
- **RESTE-T-IL UN RÉSIDU P6** ou un décompte incohérent ailleurs dans les fichiers touchés ?

## Règles de méthode

- **Vérifie au sol avant d'affirmer.** `grep -nF` pour tout motif textuel ; cite commande et
  résultat. Une hypothèse éliminée par raisonnement n'est pas une hypothèse testée.
- Tu **peux** exécuter `cargo fmt --check`, `cargo check`, `cargo clippy`, `cargo nextest run -E`,
  `bash -n`, `pdftotext`, `gh issue view` — **sous `scripts/mem-guard.sh`** pour tout travail Rust.
- ⛔ **Tu ne peux PAS lancer la suite complète ni Playwright.** Déclare-le plutôt que de supposer.
- ⛔ **N'EXÉCUTE JAMAIS `scripts/prepare-release.sh` EN ENTIER** — il écrit dans le dépôt (bump des
  `Cargo.toml`, `CHANGELOG.md`). C'est la faute commise par la passe 4. Pour l'éprouver, **extrais
  le bloc** qui t'intéresse et exécute-le dans un `bash -c` isolé.
- ⛔ **N'écris aucun fichier du dépôt, ne commite rien, ne modifie aucune issue.** Si tu éprouves un
  test par mutation, restaure l'état exact et **vérifie-le par `git status --porcelain`** avant de
  rendre ton rapport — et dis dans tes limites que tu l'as fait.
- Lis `/home/gcorbaz/devel/kesh/CLAUDE.md` — « Migration breaking policy », « Review Iteration
  Rule », « Recompter ses propres comptes rendus », « Propagation post-patch ».

## Format

Sans préambule. Par finding : identifiant (P5-n), **sévérité**, **site**, **démonstration**
(commande + sortie), **conséquence**, **remède**. Puis : (a) tableau des sévérités ; (b) la part des
findings nés de la remédiation ; (c) ce que tu as vérifié et trouvé exact ; (d) tes limites.

⚠️ **Si tu ne trouves rien au-dessus de LOW, dis-le nettement** — c'est le résultat attendu d'une
boucle qui converge, et ce serait la première fois sur cette story. **Mais ne le dis que si tu as
réellement exercé chaque axe** : la passe précédente a rendu « 0 finding » sans faire le travail,
et deux défauts réels attendaient derrière. Dans tes limites, dis **explicitement quels axes tu as
exercés et lesquels tu n'as pas pu**.
