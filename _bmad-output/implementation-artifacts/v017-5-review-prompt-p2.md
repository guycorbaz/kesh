# Story v017-5 — revue de code P2, **ciblée** (Sonnet, une lentille)

⛔ **Lecture seule stricte.** Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `npx`, ni `make`,
ni `latexmk`, ni `python3`, ni `docker`, ni `curl`, ni `claude`, ni aucun script du dépôt (`scripts/prepare-release.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`… compris), ni `sed -i`, ni `rm`, ni `pkill`, ni `killall` ; **aucune
redirection** (`>`, `>>`, `tee`), aucun fichier temporaire ; aucun `git checkout`, `switch`, `reset`, `stash`,
`rebase`. Permis : `git log`, `git show`, `git diff`, `grep`, `rg`, `sed -n`, `cat`, `ls`, `wc`, et `pdftotext f.pdf -`
vers la sortie standard seulement. Le **seul** fichier que tu écris est ton rapport :
`/home/gcorbaz/devel/kesh-gate-logs/v0175-review-p2.md`. ⛔ **Aucune pièce réelle** (`~/kesh/`, `~/kesh-test/`,
dossier de projet de Guy, Paperless).

Langue : français soigné. Contexte frais : tu n'as lu aucune passe précédente, sauf les trois rapports P1 que tu peux
ouvrir pour comprendre ce qui était demandé.

## Ce que tu relis — la seule remédiation de la revue P1

- Worktree `/home/gcorbaz/devel/kesh-0176`, branche `story/v017-5-qr-scanne`.
- Le diff : `git -C /home/gcorbaz/devel/kesh-0176 diff cafe5da7..HEAD` (code, tests, textes, manuels, fiche, registre).
- Les findings P1 : `/home/gcorbaz/devel/kesh-gate-logs/v0175-review-p1-{B,E,A}.md` (1 MEDIUM A1, 14 LOW).
- La fiche `_bmad-output/implementation-artifacts/v017-5-qr-scanne.md` (entrée « Revue P1 » du Change Log, Dev Agent
  Record) et le registre `v017-5-choix-autonomes.md` (annotations de C-v017-5-16 et -17, C-v017-5-19 neuf).
- Les journaux : `/home/gcorbaz/devel/kesh-gate-logs/v0175/` (`p1-*.log`, `mutations/M15.log`, `M16.log`).

## Axes (exerce-les tous, ou dis pourquoi non)

1. **`echelle_du_second_temps`** (C-v017-5-19) : relis la prémisse dans
   `~/.cargo/registry/src/*/pdfium-render-0.8.37/src/pdf/document/page/render_config.rs` (`apply_to_page`) ; le filtre
   ne retire-t-il le second rendu **que** quand il redonnerait l'image du premier ? Bornes : page à l'échelle 1
   exactement, page carrée plus grande que la borne, page plus grande dans un seul sens, tolérance flottante.
2. **T10 (h), (i), T3-bis** : prouvent-ils ce qu'ils annoncent ? T3-bis lit-il bien les fixtures versionnées (extraction
   du flux JPEG du PDF) et non le générateur seul ?
3. **Le générateur** : toutes les tailles contrôlées avant toute écriture ?
4. **T12** : l'assertion `restants > 0` ; reste-t-il un chemin où le test passe à vide ?
5. **Les annotations datées** (A1) : disent-elles juste au regard de `palier1.log`, `palier2.log`, `palier2d.log` ? Les
   entrées passées ont-elles été réécrites (interdit) ou seulement annotées ?
6. **La réponse à la question de B** (annotation de C-v017-5-17) : l'argument tient-il ?
7. **Textes, `.tex` et PDF** : « Sur des scans de test… une pièce réelle peut encore y échapper » (manuels, README) ; la
   phrase de l'administration sur le second rendu omis ; « trente minutes » et #863 ; la trace `reactivated` dans
   `docs/api-external.md` ; l'entrée #863 du CHANGELOG. Lis les PDF aplatis :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` ; compte par
   `timeout 60 sh -c "pdftotext f.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF 'motif' | wc -l"`, jamais un `{0,N}`.
8. **Bornes anti-DoS** et **données existantes** : rien relâché, rien réécrit d'office par la remédiation.
9. **Les commits** : `git log cafe5da7..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
   doit être vide.
10. **Ce qui a tourné** : T4 et le banc rejoués au dernier commit de code, gate complet et E2E — les chiffres du Dev
    Agent Record égalent-ils les journaux ?

## Le rapport

Pour chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), ligne lue copiée, défaut, remède.
Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée. Termine par la liste des **axes
exercés** et **non exercés** (avec la raison) et le décompte : CRITICAL n · HIGH n · MEDIUM n · LOW n. Un « 0 finding »
sans cette liste ne compte pas comme passe.
