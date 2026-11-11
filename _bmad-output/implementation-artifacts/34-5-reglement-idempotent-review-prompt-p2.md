# Revue de code P2 — 34-5 (#951), passe complète, trois lentilles B, E, A (Sonnet, lecture seule)

Tu es **une** des trois lentilles de la passe 2 de `bmad-code-review` de la story **34-5-reglement-idempotent** de Kesh
(issue #951, P1 : un second envoi du même règlement de facture client écrivait un second règlement). Modèle : **Sonnet**.
Contexte frais. Travaille en français soigné. C'est une passe **complète** : le périmètre est toute la story, et non la
seule remédiation de la P1. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche les défauts que le code porte en lui-même — un chemin
  qui écrit encore deux fois, une comparaison hors verrou, un rejeu sur interblocage qui rendrait 409 à tort, un état
  d'écran qui laisse le geste ouvert sur des montants périmés, un texte qui ment, une aide de test qui masque le contrôle.
- **E — Edge Case Hunter** : chaque branche et chaque frontière — version égale, inférieure, supérieure, absente ;
  facture annulée, brouillon, payée, inconnue, d'une autre société ; deux envois simultanés, successifs, rejeu après
  interblocage ; 409 suivi d'une relecture réussie, en échec, supplantée par un filtre, puis *Relire* ; issue inconnue
  puis 409 ; annulation d'un règlement en échec puis règlement ; échéancier avec une autre ligne ouverte pendant l'envoi.
- **A — Acceptance Auditor** : la fiche contre le code, AC par AC (AC-1 à AC-12, 11 bis), tâche par tâche (T1 à T8),
  mutation par mutation (M1 à M10, journaux ci-dessous), manuels `.tex` **et PDF**, `docs/api-external.md`, CHANGELOG,
  `scripts/recette-montee.sh`, registres, et la remédiation de la P1 point par point.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e345 log --oneline 665de151..HEAD
git -C /home/gcorbaz/devel/kesh-e345 diff 665de151 HEAD
git -C /home/gcorbaz/devel/kesh-e345 diff 992799e3 HEAD    # la remédiation de la P1, à relire en priorité
```

Lis le diff **aplati** (un seul diff de `665de151` à `HEAD`). Fichiers principaux :
`crates/kesh-db/src/repositories/invoice_settlements_write.rs` (`settle_invoice`, `expected_version`),
`crates/kesh-api/src/routes/invoices.rs` (`SettleInvoiceRequest.version`, handler),
`crates/kesh-api/tests/invoice_settlement_response_e2e.rs`, `crates/kesh-db/tests/invoice_settlement.rs`,
`crates/kesh-db/tests/support/reglement_version.rs`, `crates/kesh-api/tests/common/mod.rs`, les appelants de test,
`frontend/src/routes/(app)/invoices/[id]/+page.svelte` et `…/due-dates/+page.svelte` (états `conflict` et
`conflict-blocked`, `load()` de l'échéancier, annulation d'un règlement de la fiche), leurs Vitest,
`frontend/src/lib/features/invoices/*`, `unknown-outcome.ts`, les quatre `messages.ftl`,
`frontend/tests/e2e/invoices-settle-version.spec.ts`, `scripts/recette-montee.sh`.

Fiche : `/home/gcorbaz/devel/kesh-e345/_bmad-output/implementation-artifacts/34-5-reglement-idempotent.md` (lentille A ;
B la lit **après** son examen à l'aveugle, pour classer). Registres : `e34-choix-autonomes-e343.md` (C-e343-1 à 5 et 13)
et `e34-choix-autonomes-e345.md` (C-e345-1 à 11). Rapports de la P1 :
`/home/gcorbaz/devel/kesh-gate-logs/e345-review-p1-{B,E,A}.md`. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e345/`
(rouges `rouges.log`, mutations `mut/M*.log` et `mut/campagne*.log`, gates `chaine-p1-head.log` et `gates-p1g-*`,
frontend `p1-front-*`, E2E `e2e-r5/playwright.log`, captures `captures/` ; le Dev Agent Record les cite).

## Ce qui est décidé et ne se rediscute pas

- La version de la facture plutôt qu'une clé d'idempotence (C-e343-1), la version obligatoire (C-e343-2), aucune
  migration. Un 409 sur un rejeu, au lieu de la réponse d'origine, est le prix assumé.
- **Décision de l'orchestrateur à la P1** : le comportement de l'écran après une issue inconnue **ne change pas**.
  L'écran relit la facture et montre le règlement s'il est passé ; un clic après cette relecture est un geste délibéré,
  accepté, qu'aucune version n'empêche ni ne doit empêcher. La version refuse le renvoi **sans relecture** (second onglet,
  client de l'API, relance automatique, double envoi concurrent). Un texte qui dirait autre chose est un finding ;
  l'absence d'un refus après relecture n'en est pas un.
- **C-e345-10** : sur l'échéancier, tout chargement réussi lève l'état bloquant.
- **C-e345-11** : la fermeture du dialogue d'une autre ligne (héritée de la 33-6) n'est pas changée ici ; une issue est
  demandée.
- **M7 déclarée équivalente** : sur l'échéancier, `conflict-blocked` ne coexiste jamais avec une ligne affichée (liste
  masquée pendant le chargement, vidée sur échec, état levé sur succès). Conteste-le si tu trouves un chemin qui montre
  une ligne dans cet état.

## Axes obligatoires

1. **Un seul règlement écrit par envoi** : tout chemin par lequel deux envois à la même version écrivent deux fois —
   lecture de la version hors du `SELECT … FOR UPDATE`, comparaison après une écriture, rejeu qui relirait une version
   changée, appelant de production de `settle_invoice` autre que la route (`git grep -n "settle_invoice(" -- crates`).
2. **Ordre des refus** (AC-4) : forme, montant, puis sous le verrou facture inconnue, statut, version, date,
   contrepartie, trop-perçu, arrondi ; aucun refus n'écrit, la version reste inchangée. Le doc-comment de
   `settle_invoice` et le tableau de `docs/api-external.md` le disent-ils exactement ?
3. **Les tests prouvent-ils** : AC-3 (deux requêtes vues en attente sur le verrou), AC-2, AC-4, la version supérieure,
   la facture inconnue (404), l'arrondi témoin ; les Vitest d'E1-5, E1-7, E1-4, A1-2. Un test passe-t-il à vide ?
4. **Écran** : version envoyée (fiche : celle de la dernière réponse ou relecture ; échéancier : `cible.version`) ; 409,
   relecture, avis ; `conflict-blocked`, *Relire* ; `load()` qui lève l'état bloquant — une course où un chargement
   ancien lèverait l'état sur des données périmées ? ; annulation en échec qui relit la facture.
5. **Le manuel** — `docs/manual/fr/user-manual.tex` (§ *Enregistrer et annuler un règlement*) et
   `docs/manual/fr/admin-manual.tex` (§ *Passer à la 0.17.6*), **et leurs PDF** `docs/manual/fr/user-manual.pdf`,
   `docs/manual/fr/admin-manual.pdf` : contrôle les PDF aplatis
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `/usr/bin/grep -oF`), avec l'apostrophe
   droite **et** typographique (« Rien n'empêchant », « Rien n’empêchant ») ; disent-ils exactement la décision
   ci-dessus ? Les PDF sont-ils postérieurs au dernier `.tex` ?
6. **Contrat publié** : `docs/api-external.md` (§ *Enregistrer un règlement*), CHANGELOG (`## [0.17.6] — Non publié`,
   *Corrigé* et *Modifié* ⚠️), `scripts/recette-montee.sh`.
7. **Textes vrais** : greppe le symptôme sur tout le dépôt — toute phrase qui promettrait qu'un clic après relecture est
   refusé, ou qu'un même règlement « n'est écrit qu'une fois » sans réserve.
8. **Mutations** : M1 à M10 — chaque `M*.log` montre-t-il la mutation appliquée, le test rouge attendu, pour la bonne
   raison, et la restauration ? M7 : l'argument d'équivalence tient-il ?
9. **Captures** : `/home/gcorbaz/devel/kesh-gate-logs/e345/captures/` — les avis se voient-ils, thème clair ; le bouton
   paraît-il actif sur `fiche-conflit-apres-relire.png` ?

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur une commande
`/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou `git -C … show HEAD:<fichier> | /usr/bin/grep -nF …`), **dont tu
copies la sortie**. Bloc multi-lignes : `-A5`. (`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.)

## Sévérités

CRITICAL (un même envoi écrit deux fois, perte ou corruption de données), HIGH (AC faux, test qui ne prouve rien,
geste rouvert sur des montants périmés), MEDIUM (incohérence code/fiche/manuel/API, mutation tuée pour une mauvaise
raison, manuel faux), LOW (rédaction, nit).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`, `curl`,
`sqlx`, `mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de `git checkout` /
`switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande `gh` qui écrit ;
pas de `claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>` ni fichier
temporaire**. **Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e345-review-p2-<B|E|A>.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`B2-1`, `E2-1`, `A2-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif
   proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
