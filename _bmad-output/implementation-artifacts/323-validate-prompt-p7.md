# Prompt — validation P7, Story 32-3 (la couche de services : le motif, sur deux modules pilotes)

*Versionné le 2026-10-24. Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille, le regression
hunter**, en contexte frais, **Sonnet**, braquée sur **le seul diff de la remédiation P6**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-3`, branche `story/32-3`. **Le diff à relire, aplati, et lui seul** :

```sh
git show 26731b29 -- _bmad-output/
```

(commit `docs(32-3): validation P6 remédiée — affirmation sur le lint unsafe_code atténuée, plage du registre
corrigée`, un seul commit ; le commit suivant n'ajoute que ce prompt). Fiche :
`_bmad-output/implementation-artifacts/32-3-services-motif.md` ; registre : `epic-32-choix-autonomes-323.md` ; rapport
de la P6 qui l'a motivée : `/home/gcorbaz/devel/kesh-gate-logs/323-validate-p6.md` (0/0/1/4).

## Ce que la remédiation P6 a fait

- **P6-1 (MEDIUM), retrait** : l'effet du lint `unsafe_code` sur l'expansion d'une macro d'une autre crate n'est plus
  affirmé comme établi — « un seul précédent lu, non compilé, suggère que le lint ne se déclenche pas » —, aux deux
  sites de la fiche et aux deux du registre.
- **P6-2, P6-3, P6-4 (LOW)** : consignés au Change Log pour le développement (`cookie_secure_tests` de `config.rs` ;
  liste d'exceptions de (v-1) à nommer dans le test ; M14 si deux attributs `forbid` sont gardés).
- **P6-5 (LOW)** : `sprint-status.yaml`, plage « C-32-3-1 à 16 » portée à « C-32-3-1 à 33 ».
- Entrée « Validation P6 » ajoutée au Change Log.

## Ta tâche — chercher la régression que ce patch a introduite

*La sévérité se déplace vers ce qu'on vient d'écrire* : cherche dans **ce diff** ce qu'il casse, contredit ou affirme
à tort. **Chaque affirmation nouvelle sur le code se vérifie au code** — ouvre le site (`sed -n`, `grep -nF`) et
**copie la sortie**. En particulier :

1. **L'atténuation est-elle complète et cohérente ?** Greper sur la fiche, le registre et la clé : `contredit`,
   `établi`, `précédent`, `suggère`, `ne se déclenche pas`. Une phrase **vivante** affirme-t-elle encore, ailleurs,
   comme établi le comportement du lint sur une macro externe — ou, à l'inverse, l'atténuation contredit-elle une
   phrase voisine restée affirmative ?
2. **Les numéros de `config.rs`** de P6-2 : `grep -n "cfg(test)\|^mod \|unsafe" crates/kesh-api/src/config.rs` —
   `cookie_secure_tests` est-il bien aux lignes 3134-3286 et sous `#[cfg(test)]` ? Reste-t-il un `unsafe` hors des
   modules de test ? L'AC1 nomme-t-il désormais tous ces modules, ou seulement le Change Log (et est-ce suffisant, le
   développement lisant l'AC1) ?
3. **P6-3 et P6-4** : consignés en LOW au seul Change Log, sont-ils des exigences que le développement **doit** lire
   pour ne pas se tromper — et, si oui, leur place au Change Log suffit-elle ou faut-il un renvoi depuis l'AC10/T1 ?
4. **La plage du registre** : `grep -n "C-32-3-" _bmad-output/implementation-artifacts/sprint-status.yaml` et le
   dernier `C-32-3-` du registre — 33 est-il le bon maximum ? L'en-tête de la fiche dit-il la même chose ?
5. **Décomptes** de l'entrée « Validation P6 » : « 0/0/1/4 » contre le rapport P6 (`grep -c "^### P6-"` ou
   équivalent) ; « P6-5 (LOW) » : le rapport classait-il bien ce point LOW ? Tendance « P1 6 → … → P6 1 » contre les
   entrées du Change Log : recompte à la source.
6. **Ton** : une phrase du diff affirme-t-elle comme établi ce qui n'a été ni compilé ni lu ?
7. **Manuel** : le diff touche-t-il une affirmation sur le comportement visible ? (Attendu : non.)

Hors de ta lentille : relire la fiche entière, rejouer les axes des passes P1 à P6. Un défaut d'origine que tu
croiserais se signale, en le marquant « hors diff ».

## Ce que tu rends

Findings `P7-1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, **preuve (commande et
sortie copiée)**, correction proposée ; pour chacun, dis s'il **naît de la remédiation P6**. ⛔ Tout finding CRITICAL
ou HIGH affirmant qu'un texte « manque » ou « n'a pas été changé » se vérifie par `grep -nF` sur le fichier **final**.
Un bilan d'en-tête **cohérent avec le corps**, recompté. ⛔ **La liste des axes exercés ET non exercés** (points 1 à 7)
— sans elle, « 0 finding » ne compte pas. Rappel : la boucle ne se clôt que sur une passe qui ne rend **aucun** finding
au-dessus de LOW ; c'est la dernière passe avant le plafond de 8. Ce que la fiche **prescrit** au développement
n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet**, un seul fichier :
`/home/gcorbaz/devel/kesh-gate-logs/323-validate-p7.md`. Dernier message : chemin, bilan par sévérité, une ligne par
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
