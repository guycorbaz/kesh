# Epic 35 — Corrections triées par impact

**Kickoff : 2026-11-12**, en autonomie, pendant la construction de l'image 0.17.6 (« tu peux commencer les
développements pendant la création de l'image docker 0.17.6 »). **Cible : v0.17.7.**

*Rédigé par l'agent de kickoff (Opus 5.5), worktree `kesh-e35`, branche `chore/epic-35-kickoff` partie de `main`
(`296bbc4d`). Le code de référence est celui de **`release/0.17.6`** (`4e1c768a`) : les stories en partent, puis se
rebasent sur `main` après la fusion de la 0.17.6. Registre des choix du kickoff :
`_bmad-output/implementation-artifacts/e35-choix-autonomes-e350.md`.*

## Objet

Guy tient ses livres réels dans Kesh 0.17.5 depuis le 2026-11-10, et émettra sa **première facture client avant le
2026-11-30**. Il a demandé des epics de correction **courtes, à petites stories, triées par impact** : ce qu'un défaut
coûte dans *son* usage, non le risque abstrait. L'Epic 35 prend les huit premiers paquets du classement.

## Le critère d'impact, et sa source

Source : `kesh-gate-logs/epic35-classement-bugs.md`, §§ 7 et 8 (les plus récents font foi), établi le 2026-11-12 sur
`main` et `release/0.17.6`, puis corrigé par deux séries de réponses de Guy (mémoire `usage-reel-guy.md`).

| Niveau | Ce que le défaut coûte à Guy quand il se produit |
|---|---|
| **I1** | livres faux ou données perdues |
| **I2** | tâche bloquée, ou contournement coûteux |
| **I3** | gêne, avec un contournement simple |
| **I4** | aucun effet visible sur son travail (test, dette interne, domaine qu'il n'emploie pas, assistant déjà passé, langue autre que le français) |

L'impact est la clé de tri. La probabilité (dans son usage : saisie mois par mois, base petite, MariaDB 10.11.11,
comptes bancaires chacun lié à un compte distinct, factures client au format de numéro par défaut, agent qui écrit
par une clé d'API REST) départage à impact égal. Les 24 étiquettes `P1`–`P4` que le classement conteste ne sont **pas**
changées ici : la décision revient à Guy (rétrospective de l'Epic 34).

## Décomptes de départ

Mesurés le **2026-11-12 à 20 h 36**, par les commandes du § *Priorités des défauts* du `CLAUDE.md`, avant le tag
v0.17.6 (la PR de release n'est pas encore fusionnée) :

- **335 issues ouvertes** (`gh issue list --state open --limit 500 --json number --jq length`) ;
- **166 bugs** (label `bug` ou `known-failure`, commande du § *Priorités des défauts*) ;
- **14 P1** (`--label P1`), dont les dix de la TVA reportés.

**Au tag v0.17.6 : à mesurer au tag.** Les issues que la PR de release de la 0.17.6 ferme tomberont entre ces chiffres
et ceux du tag ; c'est le compte **au tag v0.17.6** qui sert de référence.

**Objectif** (règle de release, objectif à terme et non barrière) : **solde net en baisse** — moins d'issues ouvertes
au tag v0.17.7 qu'au tag v0.17.6, et moins de bugs, **issues ouvertes pendant l'epic comprises**. L'epic vise quatorze
issues (#431, #537, #907, #929, #932, #948, #955, #964, #1000, #1022, #1023, #1024, #1026, #1028). **On trace
toujours** : un point qu'une story ne traite pas s'ouvre en issue, une par problème, même s'il fait remonter le compte.

## Stories

| Story | Issues | Impact | Objet | Fichiers principaux (lus sur `release/0.17.6`) | Taille |
|---|---|---|---|---|---|
| **35-1** | #932, #948 | I2 | Numérotation : un format sans `{SEQ}` est refusé aux réglages ; un rendu impossible ne finit plus en 500 à la validation | `crates/kesh-core/src/invoice_format.rs` (`validate_template`), `crates/kesh-api/src/routes/company_invoice_settings.rs`, `frontend/src/routes/(app)/settings/invoicing/+page.svelte` ; peut-être `crates/kesh-db/src/repositories/invoices.rs:2334-2340` (le `map_err` en `DbError::Invariant`) | petite |
| **35-2** | #955, #964 | I2, I4 | Le refus du PDF nomme le champ et le caractère, au téléchargement comme au lot de rappels ; une adresse sans rien de visible reçoit « adresse incomplète » | `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/invoice_pdf_service.rs`, `crates/kesh-api/src/routes/invoice_email.rs` (`classify_render_error`), `frontend/src/lib/shared/utils/pdf-error.ts`, `frontend/src/lib/features/reminders/`, `messages.ftl` ×4, manuel utilisateur | petite |
| **35-3** | #537 | I1 | Les comptes désignés aux réglages de facturation (créance, TVA due, créanciers, TVA récupérable) sont revérifiés **par leur type** au moment où une facture ou une facture fournisseur y écrit | `crates/kesh-db/src/repositories/company_invoice_settings.rs` (`lock_designated_accounts_in_tx`, `check_written`), `crates/kesh-db/src/errors.rs`, `crates/kesh-api/src/errors.rs`, `frontend/src/routes/(app)/invoices/[id]/+page.svelte`, `messages.ftl` ×4, manuel utilisateur | boucle complète (règle comptable) |
| **35-4** | #1023, #1024 | I1, I3 | Réconciliation : *Rejeter* une ligne signalée « déjà comptabilisée » demande confirmation ; *Continuer quand même* dit pourquoi rien ne se passe | `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`, `AlreadyBookedConfirmDialog.svelte`, `already-booked.ts`, `messages.ftl` | petite |
| **35-5** | #929, #1022, #907 | I1, I1, I3 | La date du jour est celle de l'installation, côté serveur (contre-passation, annulation, solde d'ouverture, verrou de période, rapports) | un helper unique, puis les sites `Utc::now().date_naive()` de #929 (treize au classement, **à recompter**) : `routes/{bank_accounts,opening_balances,exports,admin}.rs`, `services/rapports.rs`, `repositories/{companies,dunning_eligibility,journal_entries,settlement_cancellation}.rs` | boucle complète |
| **35-6** | #1026, #1028 | I1, I3 | Solde d'ouverture : « 1.234 » à l'allemande n'est plus lu 1,234 franc en silence ; « 1_000 » est refusé ou affiché juste, jamais « NaN CHF » | `frontend/src/lib/features/bank-accounts/format.ts`, `frontend/src/routes/(app)/bank-accounts/+page.svelte` | petite |
| **35-7** | #1000 | I2 | `AccountAutocomplete` : un compte sorti de la liste n'est plus envoyé quand le champ paraît vide | `frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte` et ses tests | petite |
| **35-8** | #431 | I2 | L'audit attribue à la clé d'API ce qu'elle a fait, dans les dépôts qui tracent (avec l'item 11 de #892) | `crates/kesh-api/src/audit.rs` et les dépôts qui appellent `NewAuditLogEntry::user` (accounts, journal_entries, products, payment_batches, bank_profiles, credit_notes, fiscal_years, company_invoice_settings, company_dunning_settings, audit_log) | boucle complète |

**35-1 descend** (décision de Guy du 2026-11-12) : il garde le format de numéro par défaut, `F-{YEAR}-{SEQ:04}`, que le
défaut ne touche pas. Le défaut reste I2, mais sa probabilité tombe à basse.

**35-8 porte #431 et non #822** : l'agent de Guy écrit par une **clé d'API REST** (réponse à la question 1 du § 8.5) ;
chacune de ses actions est aujourd'hui attribuée à Guy dans la piste de contrôle.

**Hors de l'epic** :
- **#944** (QR-facture : ligne verticale, ciseaux, « Payable by », largeurs, type K…), proposé en option à la 35-2 : **non
  retenu**, C-e350-1. Il reste ouvert, pour l'Epic 36.
- **#911** et **#725** (sauvegarde) : sortis au § 8.4 du classement (probabilité nulle à court terme ; I3).
- **#669** : I3, en attente du relevé SQL du NAS. **#709** : en attente de l'arbitrage du critère par Guy.

## Ordre et dépendances

1. **35-2 et 35-3 en tête, en parallèle** : la première facture client de Guy part avant le 2026-11-30. Elles partagent
   trois fichiers, à des passages distincts (C-e350-4) :
   - `crates/kesh-api/src/errors.rs` : bras `InvoiceNotPdfReady` pour 35-2, bras du nouveau `DbError` pour 35-3 ;
   - `messages.ftl` ×4 : clés distinctes ;
   - `docs/manual/fr/user-manual.tex` : § *Les caractères d'une QR-facture* (l. 1251) pour 35-2, le paragraphe voisin
     de *Un compte des réglages devenu non imputable* (l. 1169) pour 35-3.
   L'intégration les unit ; le PDF du manuel se régénère une fois, après l'union.
2. Puis par impact, I1 d'abord : **35-4**, **35-6**, **35-5** ; puis **35-7**, **35-8**, **35-1**.
3. **35-8 après la fusion de 35-5** (`repositories/journal_entries.rs`) **et de 35-3**
   (`repositories/company_invoice_settings.rs`).
4. **35-1 après 35-3** si son correctif du 500 touche `repositories/invoices.rs` (le `map_err` de `:2334-2340`) ;
   sinon indépendante.
5. Les autres stories n'ont aucun fichier commun et peuvent avancer en parallèle, dans les limites des voies de la
   station (consignes communes).

Une story qui découvre un fichier commun non listé ici le dit dans son rapport avant de l'écrire.

## Règles de cet epic

- **Calendrier** : si, **au 2026-11-26**, toutes les stories ne sont pas terminées, la 0.17.7 se publie avec celles qui
  le sont (décision de Guy). 35-2 et 35-3 doivent y être.
- **Règle d'arrêt — migration** : une migration qui empêche le retour à la 0.17.6 fait **s'arrêter l'orchestrateur**, qui
  attend Guy. ⚠️ Lu à la lettre du `CLAUDE.md` (§ *Migration breaking policy*, P1), **toute** migration, breaking ou non,
  empêche le binaire précédent de démarrer (`sqlx` refuse une migration appliquée qu'il ne connaît pas) : **toute**
  migration déclenche donc l'arrêt (C-e350-5). Aucune story n'en attend ; **35-8** est la plus exposée (attribuer une
  action à une clé) : sa fiche cherche d'abord une forme sans migration, et dit avant d'écrire s'il lui en faut une.
- **Autonomie** : un choix de comportement visible prend l'option **prudente et réversible**, s'écrit au registre de la
  story (`e35-choix-autonomes-<clé>.md`) et se relève à la rétrospective.
- **Limites voisines** : comme à l'Epic 34, une story corrige les limites courtes qu'elle découvre dans sa zone, dans la
  mesure du possible ; le reste s'ouvre en issue, **une par problème**.
- **Agents** : consignes communes `kesh-gate-logs/prompts/e34-commun.md` (clé d'epic remplacée), l'outil Edit seul pour
  le code. **Production** : Kesh 0.17.5 sur le NAS, livres réels ; rien de réel dans le dépôt (fixtures synthétiques) ;
  `~/kesh/` ni lu ni écrit.
- **Validation et revue** : boucles du `CLAUDE.md` (Sonnet ↔ Opus pour les passes complètes, Haiku pour la passe
  ciblée de fin de boucle), close sur une passe sans rien au-dessus de LOW.
- **Conduite jusqu'à l'image** : mandat de Guy, en autonomie jusqu'à l'image Docker 0.17.7 vérifiée (boucles
  convergées, gates et E2E verts sur la branche finale, recette de montée verte ; sinon s'arrêter et le dire). La
  mise à jour du NAS reste le geste de Guy, sauvegarde d'abord.

## Production

Aucune migration attendue : le retour à la 0.17.6 doit rester possible sans restauration. La recette de montée
(`scripts/recette-montee.sh`) se joue depuis l'image 0.17.6 publiée, comme à chaque release.
