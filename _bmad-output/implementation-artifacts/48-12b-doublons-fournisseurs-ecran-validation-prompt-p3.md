# Prompt — validation P3 de la fiche 48-12b « Doublons de factures fournisseur — écran »

Passe **complète** (non ciblée), **modèle Opus** (la P2 était Sonnet), **contexte frais**, **lecture seule**. Réponds en français soigné.

## Ce que tu lis

- Worktree : `/home/gcorbaz/devel/kesh-48-12` (branche de la story 48-12). Lis d'abord son `CLAUDE.md` (§ *Review Iteration Rule*, § *Propagation post-patch*, § *Inventorier les sites NON RÉSOLUS*, § *Recette visuelle*, § *Issue Tracking Rule*), puis `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` et ce qu'il nomme.
- Fiche à valider : `_bmad-output/implementation-artifacts/48-12b-doublons-fournisseurs-ecran.md`.
- Fiche sœur, **à lire en entier** (contrat commun) : `_bmad-output/implementation-artifacts/48-12a-doublons-fournisseurs-serveur.md`. Fiche-mère : `48-12-reponses-perdues-fournisseurs.md`.
- Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12b-p1.md`, `validation-12b-p2.md` (et ceux de la 48-12a à côté).
- La remédiation de la P2 : le dernier commit de la branche (`git show HEAD`). Le code n'a pas changé depuis `af4e0e313` : vérifie-le (`git diff --stat af4e0e313 HEAD`).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Les remédiations de la P2, une par une** (V2-1 à V2-13 de `validation-12b-p2.md`) : prémisse lue au code, puis fermé / fermé en partie / déplacé. La fiche déclare V2-3, V2-4, V2-11 (et V2-5 en partie) **sans objet** après la décision de Guy sur l'issue inconnue : vérifie que le mécanisme qu'ils visaient (indicateur d'issue inconnue, clic qui libère le bouton, « peut-être la vôtre ») a bien disparu du texte normatif, et qu'aucun défaut équivalent ne renaît dans E6.
2. **Décision de Guy du 2026-11-19 — issue inconnue d'un enregistrement** : formulaire figé tant que l'avis s'affiche, aucun bouton de confirmation ni de renvoi, l'avis invite à vérifier dans la liste. Vérifie E6 : toutes ses entrées (envoi ordinaire, confirmation, complétion via E8, `onRefusal` qui lève) ; l'absence de toute transition vers un envoi ; le lien de la liste (`/supplier-invoices?q=<nom>` : la recherche `q` de la liste porte-t-elle bien sur le nom du fournisseur ? lis `list_filtered` et la page de liste) ; la sortie « quitter ou recharger » (la fiche la donne comme *lecture de la décision* : est-elle cohérente avec « la saisie n'est jamais vidée » ?) ; la conséquence sur un 500 (`isSettlementOutcomeUnknown`, `shared/utils/unknown-outcome.ts:65-68`).
3. **Décision de Guy du 2026-11-19 — dévalidation d'une annulée qui ferait doublon (48-12a)** : la table B (F4) suit-elle le contrat (confirmation des deux règles, `details.confirmable`, texte sans « 15-b », version affichée, avis effacé par `clearRefusals` et par toute relecture) ? Lis `supplier-invoices/[id]/+page.svelte` (`:199`, `:438`, `:444`, `:557`) et les tests M10, M10b, M14 de `supplier-unknown-outcome-page.test.ts`.
4. **La table d'états, complète** : construis toi-même la matrice **chaque état × chaque contrôle** de la table A (E1 à E9) et de la table B (F1 à F6) : champs, sélecteurs (fournisseur `:604`, projet `:839`), ajout/suppression de ligne (`:302`, `:306`), sous-champs de ligne (`:866`), scan (`:346`), panneau « nouveau fournisseur » (`:650`), `$effect` de l'échéance (`:249`), « Enregistrer », bouton de confirmation, « Modifier la saisie », liens, autres gestes de la fiche. Pour chaque case : la table dit-elle actif ou inactif ? une case muette, une transition manquante (événement possible dans un état sans sortie dite), deux lignes qui se contredisent, un état inatteignable ou sans sortie non voulue sont des findings. Vérifie aussi : la garde synchrone (états d'où l'on envoie) ; le corps gardé avant l'`await` ; qu'aucun chemin n'envoie autre chose que la saisie montrée ; que le `<fieldset disabled>` couvre réellement chaque contrôle dans la structure du composant (lis le balisage de `SupplierInvoiceForm.svelte`, `<form>` `:501`, et le panneau fournisseur) ; l'effet de `disabled` sur un `$effect` qui écrit un champ.
5. **Chaque AC et chaque test renvoie à une ligne de la table** (exigence de Guy) : vérifie la correspondance dans les deux sens (une ligne sans AC ni test est un finding) ; chaque mutation est-elle tuée par le test de sa ligne ? Les fichiers de test nommés existent-ils (sinon, la fiche le dit-elle) ?
6. **Cohérence entre les deux fiches (contrat d'API commun)** : codes, `details : { supplierInvoiceId, internalNumber, status, confirmable }` et la valeur de `confirmable` par geste et par code, `confirmNotDuplicate` sur les quatre corps (`supplier-invoices.api.ts:54-58`, `:94-99`, `imported-supplier-invoices.api.ts:58-62`, corps du `PUT`), textes et clés (écran contre serveur), règle de sortie commune. Toute différence de formulation est un finding.
7. **#985 (D2, D2bis, E8, E9)** : ordre `to_complete` puis `completed` ; filtre de `onRefusal` (`supplier-invoice-form.ts:484`, `catch` `:465`) ; `try/catch` ; `locked` ; dette de performance nommée ; angle mort du détachement.
8. **Manuel contre le code** : `docs/manual/fr/user-manual.tex` (sites cités : `:2072-2090`, `:2195`, `:2242-2265`, `:2362-2370`, `:2682`) ; **PDF compris**, aplati avant de greper : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (pipe vers `grep`, **jamais** de redirection). Chaque numéro de ligne cité est-il juste ? Le manuel actuel dit-il quelque chose que E6 rendra faux (encart « Si la réponse se perd ») sans que la fiche le désigne ? Les contrôles positifs d'AC-9 (apostrophe U+2019 dans le PDF) sont-ils exacts ? i18n : quatre locales (`crates/kesh-i18n/locales/`), `sitesTotal`, `i18n-un-repli-par-cle.test.ts`.
9. **Propagation** : grep des symptômes corrigés en P2 (`D1ter`, `D1bis`, `D1quater`, « peut-être la vôtre », « après un clic », `:888`, `:220-228`, `:452-461`) sur les deux fiches et la fiche-mère ; seules les entrées historiques du Change Log peuvent les garder.
10. **Modules, splitting, recette visuelle** : cinq modules ; signal de recyclage (un défaut de la P2 revient-il sous une autre forme, né de la table ?) ; la liste des captures couvre-t-elle chaque état neuf ou modifié ?
11. **Effet sur l'utilisateur réel** (Guy, livres réels) : un chemin mène-t-il encore à un second brouillon par un renvoi à l'aveugle ? une saisie peut-elle être perdue sans que l'écran l'ait dit ?

## Interdits

Lecture seule. N'exécute **aucune** commande qui écrit ou construit : pas de `cargo`, pas de `npm` ni `npx`, pas de `make`, aucun `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`), pas de `docker`, pas de `mariadb`, aucun `git` qui écrit (`commit`, `checkout`, `switch`, `reset`, `stash`, `add`, `push`, `worktree`, `branch -D`…), pas de `gh`, **aucune redirection** (`>`, `>>`, `tee`) ni fichier temporaire, aucune écriture sous `~/kesh/`. Lis par `cat`, `sed -n`, `grep`, `git show`, `git log`, `git diff`, `git grep`, `pdftotext … -` en pipe.

Le seul fichier que tu écris est ton rapport : **`/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12b-p3.md`**.

## Rapport

- En tête : passe, modèle, commit lu, fichiers lus, fichier écrit.
- **Verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW, recompté depuis la liste des findings, total cohérent avec la ventilation.
- Chaque finding : identifiant `V3-n`, sévérité, axe, lieu dans la fiche (ligne de table comprise), **code lu** (ligne citée), preuve, remède.
- La matrice état × contrôle de l'axe 4, telle que tu l'as reconstruite (cases muettes marquées).
- Vérifications positives.
- **Axes exercés** et **axes non exercés ou partiels**, chacun avec sa raison : un « 0 finding » sans cette liste ne compte pas.
- Signal de recyclage : déclaré ou non, avec le constat.
