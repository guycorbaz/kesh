# Revue de code P1 — 33-1b, le numéro interne à l'écran (#926), trois lentilles B, E, A (lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **33-1b** de Kesh (issue #926, P1). Contexte frais. Travaille en français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche ce que le code porte en lui-même (requête de liste, `LIKE`, tri, valeurs interpolées, état d'écran, course de la frappe temporisée).
- **E — Edge Case Hunter** : chaque branche et chaque frontière — sans numéro, numéro blanc, les deux numéros, `%` et `_` dans `q`, `q` blanc ou trop long, `sort` inconnu, `FF-2026-999` / `FF-2026-1000`, page au-delà du total, facture cliente dans les sites partagés, autre société, entrée d'audit passée.
- **A — Acceptance Auditor** : la fiche contre le code, AC par AC, ligne d'inventaire par ligne d'inventaire, mutation par mutation, manuels `.tex` **et PDF**, i18n aux quatre locales, captures.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e331 log --oneline <dernier commit de la 33-1a, au Change Log de sa fiche>..HEAD
git -C /home/gcorbaz/devel/kesh-e331 diff <ce commit> HEAD -- crates frontend docs CHANGELOG.md README.md .github
```

Lis le diff **aplati**. Fiche : `/home/gcorbaz/devel/kesh-e331/_bmad-output/implementation-artifacts/33-1b-numero-interne-ecrans.md` (lentille A ; B la lit **après** son examen à l'aveugle, pour classer). Registre : `e33-choix-autonomes-e331.md`. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e331/rouges-b.log`, mutations `/home/gcorbaz/devel/kesh-gate-logs/e331/mut/b-M*.log`, gates `/home/gcorbaz/devel/kesh-gate-logs/e331/gate-*.log`, E2E `/home/gcorbaz/devel/kesh-gate-logs/e331/e2e-*.log`. **Captures** (à regarder, non à supposer) : `/home/gcorbaz/devel/kesh-gate-logs/e331b/captures/`.

## Axes obligatoires

1. **Le manuel utilisateur contre le code et le PDF** — `docs/manual/fr/user-manual.tex` et `docs/manual/fr/user-manual.pdf`, liste close d'AC12 comprise (y compris `:1141-1146`), et le renvoi du manuel d'administration : contrôle le PDF aplati (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `/usr/bin/grep -oF`). Cherche aussi un passage que la liste a oublié.
2. **L'inventaire** : rejoue les quatre commandes de la fiche, compte les sites **non résolus** (tout site de lecture d'un numéro de facture fournisseur qui n'a pas son sort) ; toute forme `supplierInvoiceNumber ??`.
3. **Les captures** : chaque état d'AC11 existe-t-il, et l'image montre-t-elle ce que l'AC dit ?
4. **La multi-société** de `q`, du tri et de la lecture des numéros d'un lot.
5. **Le sort de l'audit du lettrage** : nouvelles entrées à la forme longue, entrées passées inchangées, `null` seulement sans aucun numéro (`C-e331-25`).
6. **Les champs existants** : `number`, `numero`, `invoiceNumber`, `supplierInvoiceNumber` ne changent ni de nom ni de valeur ; seul `details.documentNumber` d'un refus suit son message (`C-e331-20`).
7. **Le plancher MariaDB 10.7** : mentions corrigées (AC13), et aucune autre promesse restante.

## Contraintes

⛔ Lecture seule : aucune écriture hors de ton rapport, ni redirection ni fichier temporaire, aucune compilation ni exécution de test, `scripts/prepare-release.sh`, `git checkout`, `git stash` et `docker` interdits. `grep` est une fonction vers `ugrep` dans ce shell : `/usr/bin/grep` ou `git grep`.

Le rapport : `/home/gcorbaz/devel/kesh-gate-logs/e331b-review-p1-{B,E,A}.md`, selon ta lentille. Il porte :

- **les axes réellement exercés et ceux qui ne l'ont pas été**, sans quoi un « 0 finding » ne compte pas ;
- chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait lu, scénario, remédiation ;
- toute affirmation de présence ou d'absence adossée à une sortie `git grep` ou `/usr/bin/grep -nF` copiée ;
- un décompte recompté.
