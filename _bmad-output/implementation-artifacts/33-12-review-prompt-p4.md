# Prompt — revue P4 ciblée des Stories 33-12a et 33-12b : la remédiation P3

Une lentille, contexte frais, Sonnet ou Opus. Kesh (comptabilité suisse, Rust/Axum, 0.17.5) tient des livres réels. La remédiation P3 ne touche que des textes : manuel, PDF, CHANGELOG, fiches.

Relis par références : `git -C /home/gcorbaz/devel/kesh-e3312 diff 3fe12be3 e0281b8f`. Le code se lit à `e0281b8f`. Rapport P3 : `/home/gcorbaz/devel/kesh-gate-logs/e3312-review-p3.md`.

**Écris ton rapport à `/home/gcorbaz/devel/kesh-gate-logs/e3312-review-p4.md`**, et rien d'autre : lecture seule partout ailleurs.

## Interdits

Aucune autre écriture, aucune redirection ni fichier temporaire ; aucun `rm`, `pkill`, `git` qui écrit, `gh` qui écrit ; aucun interpréteur, rien qui compile ou exécute : ni `python3`, `node`, `bash -c`, `cargo`, `make`, `docker`, `mariadb`, `curl`, `claude`, `playwright`, ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs`. Permis, **tubes compris** : `git diff`, `git show`, `git log`, `git grep`, `/usr/bin/grep`, `sed -n`, `awk` sans redirection, `cat`, `ls`, `find` sans `-exec`, `wc`, `tr`, `cut`, `head`, `tail`, `sort`, `uniq`, `pdftotext <f>.pdf -`, `gh issue view`. Toute affirmation de présence ou d'absence s'appuie sur une sortie copiée. Rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS.

## Axes obligatoires

1. **Rien que des textes** : `git diff --stat 3fe12be3 e0281b8f -- crates frontend` vide.
2. **Le manuel contre le code, phrase par phrase**, paragraphe *Sur les champs qu'il imprime* de `docs/manual/fr/user-manual.tex` : les deux exceptions, tout ce qui passe par `texte_imprime` dans `crates/kesh-qrbill/src/pdf.rs` (`draw_invoice_section`, `lignes_adresse`, le message), ce que « le reste du corps » recouvre. Une troisième exception à la lettre de la phrase ?
3. **Le PDF** aplati (`/home/gcorbaz/devel/kesh-gate-logs/e3312/user-manual-33-12-p3.txt`) égale-t-il le `.tex` ; **le CHANGELOG** `[0.17.5]` dit-il la même chose ?
4. **Les comptes rendus** : entrée P3 au Change Log des deux fiches, annotations de L1 et L2 ; #956 relu ; aucune entrée passée réécrite.
5. Le symptôme : « en tête », « section de paiement », « ailleurs dans le corps » restent-ils ailleurs, dans ces textes, avec le sens corrigé ? La macro de version se règle à la release : hors périmètre.

## Le rapport

Décompte par sévérité, recompté, en tête. Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), `fichier:ligne`, preuve copiée, conséquence, remédiation proposée. **Liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe.
