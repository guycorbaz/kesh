# Revue de code P1 — 34-5 (#951), trois lentilles B, E, A (lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **34-5-reglement-idempotent** de Kesh
(issue #951, P1 : un second envoi du même règlement de facture client écrivait un second règlement). Contexte frais.
Travaille en français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche les défauts que le code porte en lui-même — un chemin
  qui écrit encore deux fois, une comparaison hors verrou, un rejeu sur interblocage qui rendrait 409 à tort, un état
  d'écran qui laisse le geste ouvert avec une version périmée, un texte qui ment, une aide de test qui masque le contrôle.
- **E — Edge Case Hunter** : chaque branche et chaque frontière — version égale, inférieure, supérieure, absente,
  négative, non entière ; facture annulée, brouillon, payée, inconnue, d'une autre société ; deux envois simultanés,
  successifs, rejeu après interblocage ; 409 suivi d'une relecture réussie, en échec, puis *Relire* ; issue inconnue
  puis 409 ; règlement réussi après un conflit ; échéancier avec une autre ligne ouverte pendant l'envoi.
- **A — Acceptance Auditor** : la fiche contre le code, AC par AC (AC-1 à AC-12, 11 bis), tâche par tâche (T1 à T8),
  mutation par mutation (M1 à M6, journaux ci-dessous), manuels `.tex` **et PDF**, `docs/api-external.md`, CHANGELOG,
  `scripts/recette-montee.sh`, registres.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e345 log --oneline 665de151..HEAD
git -C /home/gcorbaz/devel/kesh-e345 diff 665de151 HEAD
```

Lis le diff **aplati** (un seul diff de `665de151` à `HEAD`). Fichiers principaux :
`crates/kesh-db/src/repositories/invoice_settlements_write.rs` (`settle_invoice`, paramètre `expected_version`),
`crates/kesh-api/src/routes/invoices.rs` (`SettleInvoiceRequest.version`, handler),
`crates/kesh-api/tests/invoice_settlement_response_e2e.rs` (AC-1, AC-2, AC-3, AC-6),
`crates/kesh-db/tests/invoice_settlement.rs` (AC-4), `crates/kesh-db/tests/support/reglement_version.rs` et
`crates/kesh-api/tests/common/mod.rs` (aides de test), les appelants de test des trois crates,
`frontend/src/routes/(app)/invoices/[id]/+page.svelte` et `…/due-dates/+page.svelte` (états `conflict` et
`conflict-blocked`), leurs tests Vitest, `frontend/src/lib/features/invoices/*`, `unknown-outcome.ts`, les quatre
`messages.ftl`, `frontend/tests/e2e/invoices-settle-version.spec.ts` et les specs qui postent un règlement,
`scripts/recette-montee.sh`.

Fiche : `/home/gcorbaz/devel/kesh-e345/_bmad-output/implementation-artifacts/34-5-reglement-idempotent.md` (lentille A ;
B la lit **après** son examen à l'aveugle, pour classer). Registres : `e34-choix-autonomes-e343.md` (C-e343-1 à 5 et 13)
et `e34-choix-autonomes-e345.md` (C-e345-1 à 9). Journaux : rouges `/home/gcorbaz/devel/kesh-gate-logs/e345/rouges.log`,
mutations `/home/gcorbaz/devel/kesh-gate-logs/e345/mut/M*.log`, gates `/home/gcorbaz/devel/kesh-gate-logs/e345/gates-*`,
`front-*`, E2E `e2e-r*/playwright.log`, captures `/home/gcorbaz/devel/kesh-gate-logs/e345/captures/` (le Dev Agent
Record de la fiche les cite).

## Ce qui est décidé et ne se rediscute pas

La version de la facture plutôt qu'une clé d'idempotence (C-e343-1), la version obligatoire (C-e343-2), aucune
migration. Un 409 sur un rejeu après réponse perdue, au lieu de la réponse d'origine, est le prix assumé. Un défaut
**dans** ce cadre est un finding ; l'absence de clé d'idempotence n'en est pas un.

## Axes obligatoires

1. **Un seul règlement écrit** : tout chemin par lequel deux envois à la même version écrivent deux fois — lecture de la
   version hors du `SELECT … FOR UPDATE`, comparaison après une écriture, rejeu (`retry_on_deadlock`) qui relirait une
   version changée, appelant de production de `settle_invoice` autre que la route (`git grep -n "settle_invoice("`).
2. **Ordre des refus** (AC-4) : forme, montant, puis sous le verrou statut, version, date, contrepartie, trop-perçu,
   arrondi ; aucun refus n'écrit, la version reste inchangée. Le doc-comment de `settle_invoice` le dit-il exactement ?
3. **Les tests prouvent-ils** : l'AC-3 force-t-il vraiment deux requêtes en attente sur le verrou (motifs de
   `attendre_des_requetes_en_cours`, `au_moins = 2`) ? L'AC-2 et l'AC-4 comptent-ils règlements, écritures, audits et
   version ? Une aide `settle_at_current_version` employée là où un test devrait porter la version à la main ?
4. **Écran** : la fiche envoie `invoice.version` (celle de la dernière réponse) ; l'échéancier `cible.version` ; un 409
   ferme le dialogue, relit, affiche `settle-conflict` / `due-dates-settle-conflict` ; `conflict-blocked` désactive le
   geste et offre *Relire*, qui mène à `conflict` ; un autre 4xx garde le dialogue ; l'issue inconnue de la 33-6 garde
   son comportement. Les replis Svelte égalent-ils les clés `fr-CH`, et les quatre locales portent-elles les quatre clés ?
5. **Le manuel** — `docs/manual/fr/user-manual.tex` (§ *Enregistrer et annuler un règlement*) et
   `docs/manual/fr/admin-manual.tex` (§ *Passer à la 0.17.6*), **et leurs PDF** `docs/manual/fr/user-manual.pdf`,
   `docs/manual/fr/admin-manual.pdf` : contrôle les PDF aplatis
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `/usr/bin/grep -oF`) ; « Rien
   n'empêchant » doit être absent. Le manuel dit-il vrai contre le code (page rechargée, second onglet, délai dépassé) ?
6. **Contrat publié** : `docs/api-external.md` (§ *Enregistrer un règlement* : corps, `version` exigée, `422`, `409`,
   ordre des refus, codes réels `VALIDATION_ERROR` / `INVALID_INPUT`), CHANGELOG (`## [0.17.6] — Non publié`,
   *Corrigé* et *Modifié* ⚠️), `scripts/recette-montee.sh` (version lue et envoyée, refus si illisible).
7. **Textes vrais (AC-9)** : `git grep -nE "ne porte (pas de|aucune) version|Pas de .version|Aucun champ .version" --
   frontend crates` ne rend rien ; reste-t-il ailleurs (doc-comments, specs, manuels) une phrase qui dit qu'un second
   envoi écrit un second règlement ?
8. **Mutations** : chaque `M*.log` montre-t-il la mutation appliquée, le test rouge attendu, pour la bonne raison, et la
   restauration ?
9. **Captures** : `/home/gcorbaz/devel/kesh-gate-logs/e345/captures/` — l'avis de conflit de la fiche et de
   l'échéancier se voit-il, thème clair ?

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur une commande
`/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou `git -C … show HEAD:<fichier> | /usr/bin/grep -nF …`), **dont tu
copies la sortie**. Bloc multi-lignes : `-A5`. (`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.)

## Sévérités

CRITICAL (un même règlement écrit deux fois, perte ou corruption de données), HIGH (AC faux, test qui ne prouve rien,
geste rouvert avec une version périmée), MEDIUM (incohérence code/fiche/manuel/API, mutation tuée pour une mauvaise
raison, manuel faux), LOW (rédaction, nit).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`, `curl`,
`sqlx`, `mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de `git checkout` /
`switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande `gh` qui écrit ;
pas de `claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>` ni fichier
temporaire**. **Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e345-review-p1-<B|E|A>.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`B1-1`, `E1-1`, `A1-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif
   proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
