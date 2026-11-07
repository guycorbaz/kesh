# Prompt — revue P3 ciblée des Stories 33-12a et 33-12b : la remédiation P2

Une lentille, contexte frais, Sonnet ou Opus. Kesh (comptabilité suisse, Rust/Axum, 0.17.5, sans migration) tient des livres réels. Passe **ciblée sur la remédiation P2**, qui n'a touché aucune ligne exécutable de production : commentaires, tests, manuel, fiches.

Relis par références, jamais le fichier sur disque : `git -C /home/gcorbaz/devel/kesh-e3312 diff ace978ae SHA_FIN`, où `SHA_FIN` est le dernier commit de la branche `story/33-12-qr-facture`, à relever par `git log -1`, et à inscrire au rapport. Rapport P2 : `/home/gcorbaz/devel/kesh-gate-logs/e3312-review-p2.md`. Fiches : `_bmad-output/implementation-artifacts/33-12a-jeu-caracteres-qr.md`, `33-12b-adresse-imprimee-qr.md`, § *Remédiation P2*.

**Écris ton rapport à `/home/gcorbaz/devel/kesh-gate-logs/e3312-review-p3.md`**, et rien d'autre : lecture seule partout ailleurs.

## Interdits

Aucune autre écriture, aucune redirection ni fichier temporaire ; aucun `rm`, `pkill`, `git` qui écrit, `gh` ; aucun interpréteur, rien qui compile ou exécute : ni `python3`, `node`, `bash -c`, `cargo`, `make`, `docker`, `mariadb`, `curl`, `claude`, `playwright`, ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs`. Permis, **tubes compris** : `git diff`, `git show`, `git log`, `git grep`, `/usr/bin/grep`, `sed -n`, `awk` sans redirection, `cat`, `ls`, `find` sans `-exec`, `wc`, `tr`, `cut`, `head`, `tail`, `sort`, `uniq`, `pdftotext <f>.pdf -`. Toute affirmation de présence ou d'absence s'appuie sur une sortie copiée. Rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS.

## Axes obligatoires

1. **Aucun changement exécutable de production** : refais la preuve toi-même sur le diff de `crates/`, hors `#[cfg(test)] mod tests` : commentaires seuls ?
2. **P2-M1** : les trois textes et le manuel disent-ils désormais ce que fait `texte_imprime` en tête d'une facture, d'un rappel, d'un avoir ? Le symptôme « un avoir ne change pas » reste-t-il ailleurs (`git grep`, fiches, registre, CHANGELOG) ?
3. **Les tests neufs ou repris** : `borne_de_997_octets_exacte`, les 17 `Tj` après le dernier `re`, le test d'avoir : mordent-ils ? `mut/p2-resume.log`, `mut/mut-p2.py` à lire seulement.
4. **Le manuel** (`.tex`, PDF de la branche, aplati : `/home/gcorbaz/devel/kesh-gate-logs/e3312/user-manual-33-12-p2.txt`) : « deux exceptions », le lot de rappels contre `reminder-error-label.ts`, #955 et #957, le CHANGELOG.
5. **Les comptes rendus** : Change Log (#955, #956, #957), § *Remédiation P2*, décomptes recomptés, annotations datées sans réécriture d'une entrée passée.

## Le rapport

Décompte par sévérité, recompté, en tête. Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), `fichier:ligne`, preuve copiée, conséquence, remédiation proposée. **Liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe.
