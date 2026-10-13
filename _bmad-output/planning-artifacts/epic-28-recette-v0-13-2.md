# Epic 28 — Recette de la v0.13.2 : six bugs de moins, et clore un compte bancaire

**Kickoff : 2026-10-13.** La v0.13.2 est en fonction sur le NAS depuis le 2026-10-13. Guy y a
supprimé son import fautif et réimporté son relevé — **sans voir le bouton *Supprimer l'import***,
qui se lit comme du texte ([#640]). L'epic part de ce constat, des cinq bugs choisis avec Guy à la
rétrospective de l'Epic 27 (action B1), de [#633] (Q8) et d'une fonctionnalité demandée en réponse à
la Q7, la clôture d'un compte bancaire ([#642]). **Cible : v0.13.3.**

⚠️ **Contrepartie BMAD des issues** — chaque issue tient le détail, cet epic le périmètre. Il se
clôt quand les sept issues ci-dessous sont fermées et la v0.13.3 publiée.

## Le décompte, et la règle de release

```sh
gh issue list --state open --limit 500 --json labels \
  --jq '[.[]|select([.labels[].name]|any(.=="bug" or .=="known-failure"))]|length'
```

| Moment | Bugs | Source |
|---|---|---|
| Tag v0.13.2 (2026-10-13) | **54** | rétrospective de l'Epic 27, § 2.1 |
| Kickoff de l'Epic 28 (2026-10-13) | **55** | recompté à ce kickoff par la commande ci-dessus (+ [#640], recette de la v0.13.2) |
| **Plafond au tag v0.13.3** | **≤ 53** | règle de Guy, « un bug de moins à chaque release », **mesurée au tag** (Q9) |
| **Objectif** | **49** | 55 − les six bugs de l'epic |

[#642] porte le label `enhancement` : sa fermeture ne fait pas baisser le décompte. **Avant le tag**,
le nombre se recompte depuis GitHub ; un bug ouvert entre-temps — la recette en a ouvert un à chacune
des deux dernières releases — se ferme avant, ou retarde le tag s'il fait dépasser 53.

⚠️ **La règle crée une incitation**, relevée à la rétrospective de l'Epic 27 (§ 2.1, Q9) et au
registre des risques du dossier de projet (R11) : un défaut découvert **hors** de la story en cours
**s'ouvre en issue** comme le veut le § *Issue Tracking Rule*, même s'il fait remonter le compte. Ne
pas ouvrir d'issue pour tenir un chiffre n'est pas un motif recevable.

## Décisions du Project Lead (2026-10-13)

- **Autonomie complète jusqu'à l'image Docker v0.13.3** : « oui, traite l'epic 28 en toute autonomie
  jusqu'à la création de l'image docker » — push, PR, merge, tag annoté et `release.yml` compris.
- **Q4 — passe ciblée dès la P2 en revue de code** : acceptée quand la P1 n'a trouvé que des tests
  manquants ; la règle du `CLAUDE.md` (« quand une boucle converge ») reste la référence pour les
  autres cas.
- **Q5 — les actions reportées des rétrospectives** (verrou de gate, consignes des agents versionnées,
  etc.) entrent dans **cette** release : Story 28-0.
- **Q7 → [#642]** : « un compte devrait pouvoir être archivé […] s'il n'est plus utilisé ou clos dans
  la banque : on pourrait, par exemple, le marquer en lecture seule ». Ajoutée à l'epic (« ok, ajoute
  642 »).
- **Q8 → [#633]** dans l'epic : « d'accord ».
- **Q9** : la règle « un bug de moins » se mesure **au tag**.

## Les stories

Chaque story tranche ses points en autonomie, au registre `epic-28-choix-autonomes.md`, et suit les
consignes communes de `docs/agents-orchestration.md` (versé par la 28-0).

**Recette visuelle (B2 de la rétrospective de l'Epic 27).** Toute story qui ajoute ou change un
contrôle visible livre des **captures d'écran** Playwright de l'écran concerné, thème clair **et**
sombre, jointes au prompt de revue et à la PR. *Un test qui trouve un bouton ne prouve pas qu'on le
voit* : la 28-1, la 28-2, la 28-4 et la 28-5 sont concernées d'office ; la 28-6 et la 28-7 si elles
changent ce que l'écran affiche.

### 28-0 — Kickoff et actions reportées (doc seulement)

Ce fichier, le registre de l'epic, la section `epic-28` de `sprint-status.yaml`, et les actions des
rétrospectives qui ne demandent qu'un commit de documentation : consignes des agents et verrou de gate
versés dans `docs/agents-orchestration.md`, recette E2E du `CLAUDE.md` exécutable telle quelle, « dix
P1 de la TVA », propagation des décomptes (B4). Sort des actions non traitées : registre, C-28-0-k.

### 28-1 — Les boutons destructifs se voient ([#640], P3)

Variante `destructive` du `Button` partagé (`frontend/src/lib/components/ui/button/button.svelte`) :
`bg-destructive/10` sous un texte rouge, sans bordure. **17 fichiers** l'emploient (`grep -rln
'variant="destructive"' frontend/src`, recompté au kickoff).

- trancher : fond plein ou contour marqué ; contraste **WCAG AA** mesuré (texte et bordure, clair et
  sombre), états survol, focus et désactivé ;
- corriger la **variante**, non les sites ; inventorier les 17 sites et vérifier chacun **à l'écran**
  (captures avant / après), dont les dialogues où il côtoie *Annuler* en `outline` ;
- vérifier qu'aucun site n'employait la variante pour un **texte** d'erreur plutôt qu'un bouton.

### 28-2 — Une erreur d'API ne s'affiche plus « [object Object] » ([#520], P2)

`ApiError` est une interface, non une `Error` : `e instanceof Error ? e.message : String(e)` rend
« [object Object] ». **22 occurrences** de `String(e)` / `String(err)` hors tests au kickoff (`grep -rn
"String(e)\|String(err)" frontend/src --include=*.svelte --include=*.ts | grep -v '\.test\.'`), à trier
— certaines ne reçoivent jamais d'`ApiError`.

- un **utilitaire unique** de conversion d'erreur, plutôt que N correctifs ;
- **inventorier les sites non résolus** (§ *Review Iteration Rule*), pas seulement ceux du constat :
  tout motif qui affiche une erreur attrapée — `String(…)`, interpolation `${e}`, `e.message` sans
  garde —, ensemble clos et décompté ;
- le message montré est celui du serveur (déjà traduit) ; repli traduit dans les quatre locales.

### 28-3 — La sauvegarde lit ses tables dans un instantané commun ([#633], P2)

`admin_backup/export.rs` appelle `kesh_db::backup::export_table` table par table **sur le pool** :
aucune transaction, aucun instantané commun.

- une seule connexion, `START TRANSACTION WITH CONSISTENT SNAPSHOT` (ou `REPEATABLE READ` + lecture
  seule), pour toutes les tables ; effet sur la durée de verrouillage et sur le streaming ;
- **trancher le sort de [#465]** (P2), jumeau pour l'export de souveraineté (`exports/global.rs`) :
  le traiter dans la même story avec le même utilitaire (un bug de plus fermé) ou l'en séparer par
  écrit ;
- comment prouver l'instantané par un test (écriture concurrente entre deux lectures) ;
- **`kesh-db` touché → gate complet même en boucle.**

### 28-4 — Clore un compte bancaire, en lecture seule ([#642], CR) — la fonctionnalité de l'epic

Aujourd'hui l'archivage d'un compte bancaire est refusé dès qu'il porte une transaction
(`412 BANK_ACCOUNT_HAS_TRANSACTIONS`) : un compte qui a servi ne se retire jamais de l'usage.

- état distinct ou fusion avec `archived` ; **migration** (colonne nullable, date de clôture
  facultative) — lignes P5 et P7 de la *Migration breaking policy*, `min_required` à juger ;
- **gardes serveur sur TOUS les chemins d'écriture** qui prennent un `bank_account_id` — import de
  relevé (CSV, CAMT), réconciliation (`accept`, `reject`, lots), règlement de facture, lots de
  paiement, compte principal —, clés d'API comprises : **inventaire clos** des sites, non une liste
  des chemins connus ;
- sort des transactions `pending` restantes (refus de clore, ou clôture qui les laisse consultables) ;
  solde du compte comptable lié non nul (avertir ou refuser) ; réouverture tracée à l'audit ;
- lecture intacte : listes (filtre « inclure les comptes clos »), rapports, export, sauvegarde ;
  conservation de dix ans (CO 958f) ;
- manuels utilisateur et d'administration (PDF compris), i18n quatre locales.

### 28-5 — Une transaction rejetée redevient atteignable ([#526], P2)

« Rejeter » pose `auto_match_rejected_at` ; la liste des propositions les exclut, et seule
l'annulation d'un rapprochement remet le marquage à zéro.

- filtre ou onglet des rejetées, et geste « annuler le rejet » (route, rôle, trace d'audit) ;
- cohérence avec ce que viennent d'écrire la 27-1 (rejetées supprimées avec leur import) et la 27-4
  (`RECONCILIATION_ALREADY_REJECTED` dans `failed[]`) ;
- le manuel, que la 15-5c a réécrit « sans promettre de chemin », redevient affirmatif.

*Découpée à la validation P1 (2026-10-13, C-28-5-17)* : **28-5a** (serveur — route `…/unreject`, champ
`autoMatchRejectedAt`, audit ; PR `refs #526`), puis **28-5b** (écran, manuel, E2E ; PR `closes #526`), sans
release entre les deux. La limite de 100 propositions de la page *Réconciliation* est suivie par **#649**.

### 28-6 — Le rapprochement refuse un débit ([#482], P2)

`accept_one_invoice` ne vérifie pas le signe : un débit dont la référence ou le contact désigne la
facture passe la garde de score et écrirait un règlement négatif.

- `FailedProposal` dédié à l'entrée d'`accept_one_invoice` (§ *Pattern batch*), code canonique ;
- adapter `post_accept_filters_signed_amount` (qui attend `RECONCILIATION_SCORE_TOO_LOW`) ; cas d'un
  débit à référence reconnue, et d'un montant nul ;
- vérifier les autres `accept_one_*` (règle, ventilation) pour la même absence de garde.

### 28-7 — Une facture postérieure au paiement n'est plus proposée ([#548], P3)

La proposition retient ±30 jours autour de la date comptable (`repositories/reconciliation.rs`) ;
l'acceptation refuse un paiement antérieur de plus d'un jour à la facture (`value_date`
prioritaire).

- **une seule borne** partagée par la proposition et l'acceptation (même date de référence), plutôt
  que deux règles alignées à la main ;
- tests aux bornes (J−1, J, J+1, `value_date` absente) ;
- **`kesh-db` touché → gate complet même en boucle.**

## Ordre

- **28-1, 28-2, 28-3 en parallèle** : ni module ni fichier commun attendu.
- **28-5, 28-6, 28-7 sérialisées** : toutes dans `routes/reconciliation.rs` (et la 28-7 dans
  `repositories/reconciliation.rs`) — la configuration qui a fait différer la 27-4 derrière la 27-1.
  Chacune se merge avant que la suivante soit rebasée.
- **28-4 après elles, ou en parallèle avec rebase** : elle pose des gardes sur les chemins d'écriture
  de la réconciliation et de l'import, ceux-là mêmes que touchent la 28-5 et la 28-6.
- Intégration **sérielle** sur `main`, comme sur l'Epic 27 : une PR mergée avant que la suivante soit
  rebasée.

### Release v0.13.3

Après toutes les stories : `scripts/prepare-release.sh 0.13.3`, macros des manuels et URL au tag,
CHANGELOG daté, recomptes du `CLAUDE.md`, **recompte des bugs (≤ 53)**, tag annoté, `release.yml`,
vérification Docker Hub (`:0.13.3`, `:latest`, `/health`). Autorisé d'office pour cet epic.

[#465]: https://github.com/guycorbaz/kesh/issues/465
[#482]: https://github.com/guycorbaz/kesh/issues/482
[#520]: https://github.com/guycorbaz/kesh/issues/520
[#526]: https://github.com/guycorbaz/kesh/issues/526
[#548]: https://github.com/guycorbaz/kesh/issues/548
[#633]: https://github.com/guycorbaz/kesh/issues/633
[#640]: https://github.com/guycorbaz/kesh/issues/640
[#642]: https://github.com/guycorbaz/kesh/issues/642
