# Prompt — validation P6, Story 32-3 (la couche de services : le motif, sur deux modules pilotes)

*Versionné le 2026-10-24. Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille, le regression
hunter**, en contexte frais, **Haiku**, braquée sur **le seul diff de la remédiation P5**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-3`, branche `story/32-3`. **Le diff à relire, aplati, et lui seul** :

```sh
git show dcbfbd04 -- _bmad-output/
```

(commit `docs(32-3): validation P5 remédiée — prémisse tokio::join! retirée, forbid(unsafe_code) sur le crate à
essayer au développement`, un seul commit : il n'y a pas de diff multi-commit à recomposer ; le commit suivant n'ajoute
que ce prompt). Fiche : `_bmad-output/implementation-artifacts/32-3-services-motif.md` ; registre :
`epic-32-choix-autonomes-323.md` (C-32-3-33 est le choix de cette remédiation) ; rapport de la P5 qui l'a motivée :
`/home/gcorbaz/devel/kesh-gate-logs/323-validate-p5.md` (0/0/1/2).

## Ce que la remédiation P5 a fait

- **P5-1 (MEDIUM), retrait** : la prémisse « l'expansion de `tokio::join!` porte un `unsafe` que `forbid(unsafe_code)`
  refuserait » n'était pas établie (le lint ne se déclenche pas dans l'expansion d'une macro d'une autre crate ;
  précédent `testcontainers-0.26.3`). La contrainte « un service n'emploie pas `tokio::join!` » quitte l'AC1 ;
  l'attribut reste en tête de `services/mod.rs` ; son extension au crate devient un **essai de T1** (poser dans
  `lib.rs`, compiler, garder s'il passe, sinon consigner la cause réelle constatée) ; l'effet de l'attribut est dit
  « raisonné, établi par M14 au développement » (*Le motif*, AC1, AC3) ; *Ce que les gardes ne voient pas* et *Hors
  périmètre* disent « sauf si l'essai de T1 l'étend au crate » ; entrée P4 du Change Log et C-32-3-32 annotées.
- **P5-2 (LOW)** : (v-1) — `r#pool` et la coupure entre `pool` et `(` ajoutés aux formes hors d'atteinte ; faux rouges
  futurs (un `AppState::pool()`, toute méthode `.pool(` d'un autre type) et conduite (élargir la liste d'exceptions
  écrite de la garde, vide en 32-3).
- **P5-3 (LOW)** : `config.rs` — `tests` (l. 1665-1666 et suivantes) et `smtp_config_tests` (l. 3293-3596), sous
  `#[cfg(test)]`.

## Ta tâche — chercher la régression que ce patch a introduite

*La sévérité se déplace vers ce qu'on vient d'écrire* : cherche dans **ce diff** ce qu'il casse, contredit ou affirme
à tort. **Chaque affirmation nouvelle sur le code se vérifie au code** — ouvre le site (`sed -n`, `grep -nF`) et
**copie la sortie**. En particulier :

1. **Le retrait est-il complet ?** Greper la **valeur** sur la fiche, le registre et la clé de `sprint-status.yaml` :
   `join!`, `en séquence`, `vraisemblablement`, `refuserait`, `ne prend pas`, `Écartées`. Une phrase du texte
   **vivant** (hors relevés datés du Change Log et corps de choix annotés) affirme-t-elle encore que le crate entier ne
   peut pas prendre l'attribut, ou qu'un service ne peut pas employer une macro ?
2. **La tâche d'essai de T1** : la commande nommée (`cargo build -p kesh-api --all-targets`) compile-t-elle bien la lib
   **hors** `cfg(test)` (là où `cfg_attr(not(test), …)` s'applique) ? Garder l'attribut de `lib.rs` **et** celui de
   `services/mod.rs` est-il cohérent (un `forbid` redondant est-il permis) ? M14 reste-t-elle juste si l'essai passe ?
   L'essai contredit-il une autre exigence de la fiche (AC7 — liste exacte des fichiers touchés —, *Fichiers touchés
   (prévus)*, « tests existants inchangés ») ? `lib.rs` y figure-t-il déjà ?
3. **Les affirmations nouvelles sur `config.rs`** : `grep -n "cfg(test)\|^mod \|unsafe" crates/kesh-api/src/config.rs`
   — les numéros 1665-1666, 3292, 3293-3596 sont-ils justes ? Reste-t-il un `unsafe` hors de ces deux modules ?
4. **(v-1) et ses formes** : `r#pool` — `::\s*pool\b` ou `\.\s*pool\s*\(` le reconnaît-il vraiment ou non (raisonne
   sur le motif, sans exécuter de moteur de regex hors `grep`) ? La « liste d'exceptions écrite (vide en 32-3) » :
   la fiche décrit-elle ailleurs une liste d'exceptions pour (v-1), ou l'invente-t-elle ici sans l'avoir posée à l'AC10
   ou au test ? Si elle l'invente, est-ce une exigence nouvelle mal amarrée ?
5. **Ton** : une phrase du diff affirme-t-elle comme établi ce qui n'a été ni compilé ni lu (le comportement du lint
   sur une macro externe, la validité de l'attribut, l'issue de l'essai) ?
6. **Décomptes** du Change Log « Validation P5 » : « 0/0/1/2 », « trois findings, tous nés de la remédiation P4 »
   (`grep -c "^### P5-"` sur le rapport P5) ; tendance « P1 6 → P2 5 → P3 1 → P4 1 → P5 1 » contre les entrées P1 à
   P4 du Change Log : recompte à la source.
7. **Registre** : C-32-3-33 est-il cohérent avec l'annotation de C-32-3-32 et avec la fiche ? L'en-tête de la fiche
   dit-il « C-32-3-1 à C-32-3-33 » ?
8. **Manuel** : le diff touche-t-il une affirmation sur le comportement visible ? (Attendu : non, AC14.) Si oui,
   vérifie le PDF aplati (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`).

Hors de ta lentille : relire la fiche entière, rejouer les axes des passes P1 à P5. Un défaut d'origine que tu
croiserais se signale, en le marquant « hors diff ».

## Ce que tu rends

Findings `P6-1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, **preuve (commande et
sortie copiée)**, correction proposée ; pour chacun, dis s'il **naît de la remédiation P5**. ⛔ Tout finding CRITICAL
ou HIGH affirmant qu'un texte « manque » ou « n'a pas été changé » se vérifie par `grep -nF` sur le fichier **final**
avant d'être écrit — ne te fie pas aux numéros de ligne du diff. Un bilan d'en-tête **cohérent avec le corps**,
recompté. ⛔ **La liste des axes exercés ET non exercés** (points 1 à 8 ci-dessus) — sans elle, « 0 finding » ne compte
pas. Rappel : la boucle ne se clôt que sur une passe qui ne rend **aucun** finding au-dessus de LOW. Ce que la fiche
**prescrit** au développement n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet**,
un seul fichier : `/home/gcorbaz/devel/kesh-gate-logs/323-validate-p6.md`. Dernier message : chemin, bilan par
sévérité, une ligne par finding MEDIUM ou plus.

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
