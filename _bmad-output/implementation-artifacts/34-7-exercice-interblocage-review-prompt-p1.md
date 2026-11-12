# Revue de code P1 — 34-7-exercice-interblocage (#945), trois lentilles B, E, A (lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **34-7** de Kesh (issue #945 : le
renommage d'un exercice, `PUT /api/v1/fiscal-years/{id}`, rejoué sur interblocage). Contexte frais. Travaille en
français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche les défauts que le code porte en lui-même : rejeu d'une
  erreur qui ne devait pas l'être, conversion en `AppError` dans la fermeture (rejeu muet), effet de bord hors
  transaction, audit écrit deux fois, valeur consommée par déplacement non clonée, test qui passe à vide.
- **E — Edge Case Hunter** : chaque branche et chaque frontière — nom vide, trop long, égal au sien, homonyme dans une
  autre société, exercice clos, exercice absent ou d'une autre société (404 avant toute enveloppe ?), trois interblocages
  de suite (épuisement → 500), un refus métier après un rejeu, une 1213 levée après le `commit` ; montage des tests (la
  route peut-elle être vue en attente avant de tenir M ? la lourde peut-elle devenir la victime ? dates de la ligne
  insérée de l'AC-2 contre `uq_fiscal_years_company_start_date` et l'exercice 2020-2030 du semis).
- **A — Acceptance Auditor** : la fiche contre le code, AC par AC (AC-1 à AC-4), tâche par tâche (T1 à T4), mutation par
  mutation (M1, M2, journaux cités ci-dessous), registre des routes, `docs/api-external.md`, CHANGELOG, registre des choix.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e347 log --oneline 665de151..HEAD
git -C /home/gcorbaz/devel/kesh-e347 diff 665de151 HEAD -- crates docs CHANGELOG.md
```

Lis le diff **aplati** (un seul diff de `665de151` à `HEAD`). Fichiers :
`crates/kesh-api/src/routes/fiscal_years.rs` (`update_fiscal_year`), `crates/kesh-api/tests/rejeu_interblocage_e2e.rs`
(deux tests neufs en fin de fichier, `fiscal_year_rename_is_replayed_when_it_is_the_deadlock_victim` et
`fiscal_year_valid_rename_succeeds_after_the_replay`, aide `audits_renommage`),
`crates/kesh-api/tests/audit_route_registry.rs` (prose des points (iii bis), (iv), (vi)), `docs/api-external.md`
(paragraphe « Interblocages »), `CHANGELOG.md` (section `[0.17.6] — Non publié`).

Code lu pour juger, non modifié : `crates/kesh-db/src/repositories/fiscal_years.rs` (`update_name`), `crates/kesh-db/src/retry.rs`
(`retry_on_deadlock`, `retry_on_deadlock_with`), `crates/kesh-api/tests/common/capture_rejeu.rs`,
`crates/kesh-db/src/test_fixtures.rs` (`attendre_une_requete_en_cours`).

Fiche : `/home/gcorbaz/devel/kesh-e347/_bmad-output/implementation-artifacts/34-7-exercice-interblocage.md` (lentille A ;
B la lit **après** son examen à l'aveugle, pour classer). Registres : `e34-choix-autonomes-e343.md` (C-e343-8) et
`e34-choix-autonomes-e347.md` (C-e347-1 à 3). Journaux : rouges `/home/gcorbaz/devel/kesh-gate-logs/e347/rouges.log`
et `rouges-run.log`, mutations `/home/gcorbaz/devel/kesh-gate-logs/e347/mut/M1.log` et `M2.log`, gates
`/home/gcorbaz/devel/kesh-gate-logs/e347/` (le Dev Agent Record de la fiche les cite).

## Axes obligatoires

1. **Le rejeu est sûr** : `update_name` est-il une transaction unique, audit compris ? Ses refus métier sortent-ils en
   `DbError::Invariant` (non rejoués) ? `map_update_error` est-il appliqué **après** l'enveloppe ? Le nom est-il cloné à
   chaque tentative ?
2. **Les tests prouvent ce qu'ils disent** : la route est-elle vue en attente **après** avoir pris M (motifs
   `AND name = ?` et `FOR UPDATE`, propres à la lecture de doublon) ? Le témoin du rejeu est-il exigé ? L'AC-2 prouve-t-il
   qu'une ligne non validée bloque la lecture de doublon, et que la ligne disparaît (compte d'homonymes) ?
3. **Mutations** : M1 (enveloppe retirée) et M2 (une seule tentative) sont-elles tuées par les deux tests, pour la bonne
   raison (lis `mut/M1.log` et `mut/M2.log` : 500 et aucun témoin) ?
4. **Registre des routes** : le point (iv) ne nomme plus le renommage comme candidat ; le point (vi) dit vingt-cinq —
   recompte-le toi-même depuis la table (`SansEcritureAuJournal`) et le corps des handlers ; `assert_eq!(examinees, 25)`
   n'a **pas** été touché (il compte la colonne `Rejouee`, coïncidence de valeur).
5. **Textes** : doc-comment de la route, `docs/api-external.md` (la route est-elle bien ouverte aux clés ? le `400` dit
   est-il celui que rend le code ?), CHANGELOG (vrai contre le code, sans promesse en trop).
6. **Manuel** : la story ne change aucun écran ni aucun comportement décrit aux manuels. Vérifie-le :
   `/usr/bin/grep -nF "interblocage" docs/manual/fr/user-manual.tex docs/manual/fr/admin-manual.tex` et, pour le PDF,
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -oiF "interblocage"`
   (même chose pour `user-manual.pdf`). Un manuel qui décrirait un `500` au renommage serait un finding.
7. **Captures** : aucune n'est livrée, la story n'ajoutant ni ne modifiant aucun contrôle visible. Juge si c'est exact.

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur une commande
`/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou `git -C /home/gcorbaz/devel/kesh-e347 show HEAD:<fichier> | /usr/bin/grep -nF …`),
**dont tu copies la sortie**. Bloc multi-lignes : `-A5`. (`grep` est une fonction vers `ugrep` : utilise
**`/usr/bin/grep`**.)

## Sévérités

CRITICAL (donnée faussée, audit perdu ou doublé), HIGH (AC faux, test qui ne prouve rien, rejeu d'une erreur métier),
MEDIUM (incohérence code/fiche/registre/doc, mutation tuée pour une mauvaise raison), LOW (rédaction, nit).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `make`, `latexmk`, `docker`, `curl`, `sqlx`,
`mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de `git checkout`/`switch`,
`git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande `gh` qui écrit ; pas de
`claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>` ni fichier temporaire**.
**Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e347-review-p1-<B|E|A>.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`B1-1`, `E1-1`, `A1-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif
   proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
