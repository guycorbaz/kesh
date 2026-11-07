# Prompt — revue de code P2 ciblée des Stories 33-12a et 33-12b : la remédiation P1

Une lentille, contexte frais, Sonnet ou Opus. Kesh (comptabilité suisse, Rust/Axum, 0.17.5, sans migration) tient des livres réels. Passe **ciblée sur la remédiation** de la revue P1, non sur les stories entières : la sévérité se déplace vers ce qu'on vient d'écrire.

Relis par références, jamais le fichier sur disque : `git -C /home/gcorbaz/devel/kesh-e3312 diff b19042f3 80bc4363`. Fiches : `_bmad-output/implementation-artifacts/33-12a-jeu-caracteres-qr.md`, `33-12b-adresse-imprimee-qr.md`, § *Remédiation P1*, registre `e33-choix-autonomes-e3312.md`, rapports `/home/gcorbaz/devel/kesh-gate-logs/e3312{a,b}-review-p1-{B,E,A}.md`. Rapport : **`/home/gcorbaz/devel/kesh-gate-logs/e3312-review-p2.md`**, et rien d'autre.

## Interdits

Lecture seule. **Aucune redirection, aucun fichier temporaire** ; aucun `rm`, `pkill`, `git` qui écrit, `gh` ; aucun interpréteur, rien qui compile ou exécute : ni `python3`, `node`, `bash -c`, `cargo`, `make`, `docker`, `mariadb`, `curl`, `claude`, `playwright`, ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs`. Permis : `git diff`, `git show`, `git log`, `git grep`, `/usr/bin/grep`, `sed -n`, `awk` sans redirection, `cat`, `ls`, `find` sans `-exec`, `wc`, `pdftotext <f>.pdf -`. Toute affirmation de présence ou d'absence s'appuie sur une sortie copiée. Rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS.

## Axes obligatoires

1. **Le corps par `texte_imprime`** : rogner une ligne d'adresse du corps change-t-il une mise en page, une garde de hauteur (`TooManyLines`, en-tête) ? Un avoir : U+2019 devient `'` en tête d'un avoir, est-ce dit, est-ce juste ? Les autres textes du corps restent-ils tels que le manuel le dit ?
2. **`couper_ligne` à un seul chemin** : aucune perte, ligne vide, `Ĳ`, frontières ; ce que dit AC3 contre ce que fait le code, et ce que le manuel en dit.
3. **Le message d'erreur à une seule graphie** : `err @ QrBillError::InvalidCharset` contre l'ancien texte, ses lecteurs (écran, e-mail, rappels, tests).
4. **Le message d'invisibles** : `y` et intitulé ; le code QR reste-t-il juste ?
5. **Les tests qui prouvent moins** : `operations_du_pdf`, suite des `rg` et dernier `re`, E2E qui lit le contenu, borne de 997 octets ; `mut/p1-resume.log`, et `mut/mut-p1*.py` à lire seulement.
6. **Le manuel** (`docs/manual/fr/user-manual.tex`, PDF aplati : `/home/gcorbaz/devel/kesh-gate-logs/e3312/user-manual-33-12-p1.txt`) et le CHANGELOG `[0.17.5]` contre le code et l'écran : `captures/ecran-refus-caractere.png`, `captures/p1-insecable.png`.
7. **Les PDF** : `admin-manual.pdf` et `marketing-brochure.pdf` rendus à leur état ; rien d'autre n'a bougé ?

## Le rapport

Décompte par sévérité, recompté, en tête. Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), `fichier:ligne`, preuve copiée, conséquence, remédiation proposée. **Liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe.
