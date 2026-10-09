# Prompt — revue de code P3 **ciblée**, Story 15-1a2-i (le lettrage des pièces clientes)

*Versionné le 2026-10-09. UNE lentille (Haiku), contexte frais, lecture seule — passe ciblée de fin de boucle
(`CLAUDE.md` § « La passe ciblée »).*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-15-1a2-i`, branche `story/15-1a2-i-lettrage-des-pieces-clients`.
**Le seul diff à revoir** : `git diff 7b27bf99 87f367df` — la remédiation de la revue P2. Ne revois pas le reste de
la story. Le Change Log « Revue de code P2 » de la fiche
`_bmad-output/implementation-artifacts/15-1a2-i-lettrage-des-pieces-clients.md` dit, finding par finding, ce que ce
commit prétend corriger ; les rapports P2 sont `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-i-review-p2-{B,E,A}.md`.

## La lentille — chasseur de régressions de la remédiation

Pour chaque fichier du diff :

1. **Le correctif fait-il ce que le Change Log annonce ?** (A2-1 : les trois sites — `CHANGELOG.md`, `docs/api-external.md`
   § « Lettrer des lignes », `docs/manual/fr/user-manual.tex` paragraphe *Lettrage* — disent-ils que le lettrage se pose
   au geste et qu'une facture déjà soldée avant la mise à jour n'est pas lettrée ? le cas « période close » est-il
   retiré partout ? le reçu de la fiche `15-1a2-ii-fournisseurs-et-rattrapage.md` nomme-t-il les trois sites ?)
2. **Le correctif est-il VRAI du code ?** Relis le code cité pour chaque phrase neuve : manuel (« un lettrage posé avant
   le changement subsiste, et l'annulation d'un règlement le défait » ; « avoir émis avant la version 0.13 » ;
   « créditée et réglée (avant la version 0.12.1) »), doc-comments de `letterings.rs` (§ « Interblocages résiduels »,
   étape 5, tolérance E2-1), `api-external.md` (« dé-rapprochement compris »).
3. **A-t-il cassé quelque chose ?** `letterings_lexical.rs` (`ecritures_et_suite` : même comportement que le code
   qu'elle remplace ? un `INSERT` hors fonction est-il encore signalé ?) ; `rejeu_interblocage_e2e.rs` (le témoin
   annulé : l'état final satisfait-il encore AC5 ? le décompte `lettering.removed` filtre-t-il bien par facture ?) ;
   `reconciliation_e2e.rs` ; le test du mappage.
4. **Propagation** : greppe la VALEUR de chaque symptôme corrigé sur tout le dépôt (`git grep -nE "après coup|échoue
   jamais|n.échoue JAMAIS|cycle .*neuf"` sur `crates`, `docs`, `CHANGELOG.md`, `README.md`) et dis ce qui reste.
5. **Le PDF** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (sortie sous
   `/home/gcorbaz/devel/kesh-gate-logs/` seulement) — le paragraphe *Lettrage* du PDF est-il celui du `.tex` ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-i-review-p3-ciblee.md`. Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, preuve. ⛔ Tout finding affirmant qu'un code est absent ou présent porte la
**sortie d'un `grep -nF` copiée** ; un finding qui reproche l'absence d'un livrable doit citer la ligne de la fiche qui
le prescrit **pour ce commit**. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte
pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport (et du texte aplati du PDF) dans `/home/gcorbaz/devel/kesh-gate-logs/` ;
aucune commande qui écrit, compile ou exécute : `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`),
`make`, `latexmk`, `cargo`, `npm`, `npx`, `docker`, `sqlx`, SQL, `git commit`/`add`/`checkout`/`switch`/`stash`/
`reset`/`rebase`, `gh` en écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `pdftotext` vers
`/home/gcorbaz/devel/kesh-gate-logs/`.
