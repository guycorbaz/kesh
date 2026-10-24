# Prompt — validation P5, Story 32-3 (la couche de services : le motif, sur deux modules pilotes)

*Versionné le 2026-10-24. Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille, le regression
hunter**, en contexte frais, **Sonnet**, braquée sur **le seul diff de la remédiation P4**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-3`, branche `story/32-3`. **Le diff à relire, aplati, et lui seul** :

```sh
git show a2eb9e6f -- _bmad-output/
```

(commit `docs(32-3): validation P4 remédiée — garde de l'accesseur élargie aux appels qualifiés, forbid(unsafe_code)
sur services`, un seul commit : il n'y a pas de diff multi-commit à recomposer ; le commit suivant n'ajoute que ce
prompt). Fiche : `_bmad-output/implementation-artifacts/32-3-services-motif.md` ; registre :
`epic-32-choix-autonomes-323.md` (C-32-3-32 est le choix de cette remédiation) ; rapport de la P4 qui l'a motivée :
`/home/gcorbaz/devel/kesh-gate-logs/323-validate-p4.md` (0/0/1/4).

## Ce que la remédiation P4 a fait

- **P4-1 (MEDIUM)** : la garde (v-1) de l'AC10 ne cherchait que `.pool()` ; elle cherche désormais, sur les lignes de
  code de tout `crates/kesh-api/src/` hors de `services/`, le motif `\.\s*pool\s*\(|::\s*pool\b` (méthode, appel
  qualifié `Lecture::pool(&x)`, chemin `.map(Lecture::pool)`) ; témoins positifs et négatif ; mutation **M11-bis** ;
  formes restées hors d'atteinte écrites à *Ce que les gardes ne voient pas*.
- **P4-2 (LOW), décision de l'orchestrateur** : `#![cfg_attr(not(test), forbid(unsafe_code))]` en tête de
  `services/mod.rs` (AC1) ; mutation **M14** (`transmute` dans `services/` → la lib ne compile pas) ; le crate entier
  écarté (`tokio::join!` dans `routes/invoices.rs:1156`) ; `unsafe`/`transmute` ne restent à *Ce que les gardes ne
  voient pas* que **hors** de `services/` ; C-32-3-31 annoté.
- **P4-3 à P4-5 (LOW)** : #431 et L2 (« tracé sur #431 ») ; retrait des `/* … */` sur plusieurs lignes en (vii) ;
  chemin complet de `test_fixtures.rs:118` et table paramétrée par des symboles en (b′).

## Ta tâche — chercher la régression que ce patch a introduite

*La sévérité se déplace vers ce qu'on vient d'écrire* : cherche dans **ce diff** ce qu'il casse, contredit ou affirme
à tort. **Chaque affirmation nouvelle sur le code se vérifie au code** — ouvre le site (`sed -n`, `grep -nF`) et
**copie la sortie**. En particulier :

1. **Le motif de (v-1)** : `grep -rnE "\.\s*pool\s*\(|::\s*pool\b" crates/kesh-api/src` rend-il bien **rien**
   (baseline) ? Le motif attrape-t-il les trois formes annoncées (`lecture.pool()`, `Lecture::pool(&l)`,
   `<Lecture>::pool(&l)`, `.map(Lecture::pool)`) et laisse-t-il `state.pool`, `self.pool.clone()` ? Reste-t-il une
   forme **qui compile**, d'une seule ligne, mise en forme par rustfmt, qui échappe au motif et que *Ce que les gardes
   ne voient pas* ne cite pas ? Un faux rouge possible (`sqlx::pool::…`, un identifiant `pool` dans un autre type) :
   la fiche le dit-elle juste ? Le motif est écrit en ERE pour `grep` ; le test sera écrit en Rust (crate `regex` ou
   recherche de sous-chaînes) : la fiche impose-t-elle quelque chose qui ne s'exprime pas ?
2. **Le `forbid(unsafe_code)` de module** :
   - un attribut interne `#![cfg_attr(not(test), forbid(unsafe_code))]` est-il **valide** en tête d'un `mod.rs`
     (module non racine) et s'applique-t-il bien à ses sous-modules ? Raisonne sur la référence Rust ; ne compile pas.
   - « sous `cfg(test)`, il ne s'applique pas » : la mutation M14 rougit-elle dans le gate qu'elle nomme
     (`cargo build -p kesh-api --lib`, binaire `services_motif`) ?
   - l'affirmation sur `tokio::join!` : `routes/invoices.rs:1156` l'emploie-t-il ? L'expansion porte-t-elle `unsafe`
     (lis `~/.cargo/registry/src/*/tokio-1.52.3/src/macros/join.rs`, l. 170-200, et la version dans `Cargo.lock`) ?
     La fiche dit « vraisemblablement » : est-ce assez prudent, ou trop ?
   - le motif impose-t-il désormais aux services une contrainte (pas de `tokio::join!`) qui contredit une autre
     exigence de la fiche (ordre des lectures, AC4, AC5) ?
   - `config.rs` : les `unsafe` sont-ils **tous** dans `#[cfg(test)] mod tests` (l. 1665-1666 et suivantes) ?
3. **M11-bis et M14** : M11-bis compile-t-elle (`Lecture::pool(&state.lecture())` — `state.lecture()` rend une valeur
   temporaire : la référence vit-elle assez longtemps pour l'appel ?) et rougit-elle (v-1) seule ? M14 : la fonction
   proposée est-elle bien refusée par `forbid`, et non par une autre erreur (tailles des types, `transmute` entre deux
   tuples d'un même champ) ?
4. **Cohérence de la propagation** : *Le motif*, AC1, AC3, AC10 (v-1) et *Ce que les gardes ne voient pas*, R4, R8,
   *Hors périmètre*, T5, M11/M11-bis, M14, registre C-32-3-32 et la clé de `sprint-status.yaml` disent-ils la même
   chose ? Greper la **valeur** : `\.pool\(`, `forbid`, `transmute`, `unsafe`, `M1 à M1`, `demandé à l'orchestrateur`,
   `ne couvre que`, `ne nomme que`, `test_fixtures.rs`. Une phrase du texte **vivant** (hors relevés datés du Change
   Log P1 à P3 et corps des choix annotés) affirme-t-elle encore que `unsafe` dans `services/` n'est pas vu, ou que
   (v-1) ne cherche que `.pool()` ?
5. **(vii) et (b′)** : la règle de retrait des blocs sur le corps entier reste-t-elle cohérente avec « l'absence se
   cherche sur les lignes de code » ? La table paramétrée par symboles est-elle compatible avec « constante unique »
   et avec la ligne 12 (`du`/`au`, pas `debut`/`fin`) ?
6. **Décomptes** du Change Log « Validation P4 » : « 0/0/1/4 », « cinq findings, tous nés de la remédiation P3 »
   (`grep "^### P4-"` sur le rapport P4) : recompte à la source.
7. **#431** : `gh issue view 431 --json comments` — le commentaire du 2026-10-24 existe-t-il, et le fil couvre-t-il
   bien des sites de route (`unvalidate`) ?
8. **Manuel** : le diff touche-t-il une affirmation sur le comportement visible ? (Attendu : non, AC14.) Si oui,
   vérifie le PDF aplati (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`).

Hors de ta lentille : relire la fiche entière, rejouer les axes des passes P1 à P4. Un défaut d'origine que tu
croiserais se signale, en le marquant « hors diff ».

## Ce que tu rends

Findings `P5-1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, **preuve (commande et
sortie copiée)**, correction proposée ; pour chacun, dis s'il **naît de la remédiation P4**. ⛔ Tout finding CRITICAL
ou HIGH affirmant qu'un texte « manque » ou « n'a pas été changé » se vérifie par `grep -nF` sur le fichier **final**
avant d'être écrit. Un bilan d'en-tête **cohérent avec le corps**, recompté. ⛔ **La liste des axes exercés ET non
exercés** (points 1 à 8 ci-dessus) — sans elle, « 0 finding » ne compte pas. Rappel : la boucle ne se clôt que sur une
passe qui ne rend **aucun** finding au-dessus de LOW. Ce que la fiche **prescrit** au développement n'existe pas
encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet**, un seul fichier :
`/home/gcorbaz/devel/kesh-gate-logs/323-validate-p5.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `rustfmt`, `npm`,
`npx`, `docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est en
lecture seule. Autorisés : lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `gh issue view|list`,
`pdftotext … -`, et les filtres en lecture seule `tr`, `sort`, `cut`, `head`, `awk`, `wc`, `uniq` vers la sortie
standard.
