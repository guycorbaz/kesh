# Epic 65 — Nettoyage (suite de l'Epic 58) : textes d'interface, cosmétique des écrans, tests de la réconciliation

*Créé le 2026-11-28 par réduction de l'Epic 58 (décision du Project Lead du 2026-11-27 : l'Epic 58 se clôt avec la release v0.20.1 sur ses stories 58-1 à 58-3 ; les stories 58-4, 58-5 et 58-6 passent ici).* Branche `epic/65-nettoyage-suite`, à créer depuis `main` au lancement de sa première story. Release visée : **v0.21.0, vendredi 2026-12-04**. L'epic n'entre dans `main` que terminé. Même règle que l'Epic 58 : cinq à dix issues P3 ou P4 peu coûteuses d'un même module par story, sans règle métier changée, ni migration, ni changement d'API ; boucle de revue et gates propres à chaque story ; pas de CHANGELOG écrit dans les stories ; aucun PDF commité. La 58-4 et la 58-5 touchent des textes et du CSS visibles : recette visuelle exigée.

## Stories (clés conservées)

| Story | Module | Issues | Plus ancienne |
|---|---|---|---|
| 58-4 | Textes d'interface et catalogues i18n | 10 | #324 |
| 58-5 | Cosmétique des écrans | 10 | #253 |
| 58-6 | Tests frontend de la réconciliation | 10 | #1078 |

Total : **30 issues** ; la sélection, les correctifs attendus et les gates de chaque story sont ceux du fichier de l'Epic 58 à la date du 2026-11-26, repris ci-dessous. Les stories sont indépendantes entre elles.

### Story 58-4 — Textes d'interface et catalogues i18n

**Module** : `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`, `frontend/src/routes/(app)/+layout.svelte`, `JournalEntryForm.svelte`, liste des factures.

| # | Prio | Créée | Titre | Correctif attendu | Coût |
|---|---|---|---|---|---|
| #324 | P3 | 2026-08-20 | [KF-042] Le faux ami « Valider » — journal-entry-form-submit dit Speichern/Salva/Save | Clé du bouton d'écriture : un libellé distinct de « Enregistrer » dans les 4 langues | S |
| #339 | P4 | 2026-08-23 | [KF-046] Libellés français en dur dans le MARKUP — l'angle mort qui reste après l'Epi | Libellés français en dur dans le balisage (liste des factures, puis relevé du même motif) | M |
| #844 | P4 | 2026-10-30 | [DETTE] Clé i18n morte : homepage-bank-no-transactions | Supprimer la clé morte `homepage-bank-no-transactions` (4 locales) | S |
| #861 | P4 | 2026-10-31 | [bug] Coque : le placeholder de recherche et le pied de page restent en français dans | Placeholder « Rechercher... » et pied de page via `i18nMsg` | S |
| #873 | P4 | 2026-11-03 | Refus LATER_FISCAL_YEAR_CLOSED : l'exercice nommé contredit la consigne « en commença | `LATER_FISCAL_YEAR_CLOSED` : nom de l'exercice cohérent avec la consigne | S |
| #874 | P4 | 2026-11-03 | it-CH : forme d'adresse mêlée (« tu » et « voi ») dans les refus | it-CH : une seule forme d'adresse dans les refus | S |
| #908 | P4 | 2026-11-05 | PERIOD_LOCKED : les messages du serveur affichent des dates ISO | `PERIOD_LOCKED` : dates au format suisse dans les messages | S |
| #1049 | P4 | 2026-11-12 | fr-CH : apostrophe droite dans le message RETAINED_EARNINGS_LINE (« d'ouverture ») | fr-CH : apostrophe typographique dans `RETAINED_EARNINGS_LINE` | S |
| #1086 | P4 | 2026-11-13 | Catalogue fr-CH : aucune espace insécable autour des guillemets « », qui se retrouven | fr-CH : espaces insécables autour des guillemets « » | S |
| #1491 | P4 | 2026-11-23 | Audit : le libellé « Doublon de facture fournisseur confirmé » dit le contraire du ge | Libellé d'audit « Doublon de facture fournisseur confirmé » : dire le geste | S |

**10 issues.** **Gates** : gate frontend (`check`, `lint-i18n-ownership`, `test:unit`, `build`) ; compteur `sitesTotal` de `i18n-keys.test.ts` à recompter ; gate Rust ciblé sur `kesh-i18n` ; E2E complet à la clôture (libellés) ; recette visuelle : captures de l'écran d'écriture, de la liste des factures et de l'en-tête.

### Story 58-5 — Cosmétique des écrans : colonnes collées, débordements, contraste

**Module** : `frontend/src/routes/(app)/audit-log`, `lib/features/bank-import`, `reconciliation`, `accueil`, `app.css`.

| # | Prio | Créée | Titre | Correctif attendu | Coût |
|---|---|---|---|---|---|
| #253 | P4 | 2026-07-15 | [KF] Contraste AA insuffisant sur les chips sélectionnés (bg-primary-light + texte fo | Contraste AA des chips sélectionnés (3,97:1 → 4,5:1) | S |
| #933 | P4 | 2026-11-06 | [bug] Aperçu de l'import bancaire : les colonnes Montant et Détails se touchent | Marge entre « Montant » et « Détails » (aperçu d'import) | S |
| #996 | P3 | 2026-11-10 | Contraste : --color-warning (#d97706) sur blanc mesure 3,19:1, sous le seuil AA pour  | `--color-warning` à 3,19:1 : assombrir le jeton | S |
| #1008 | P4 | 2026-11-11 | Accueil, tuile Comptes bancaires : le lien « Solde non disponible » chevauche le nom  | Tuile « Comptes bancaires » : le lien « Solde non disponible » chevauche le nom | S |
| #1015 | P4 | 2026-11-11 | Société, avis de conflit : une valeur longue sans espace déborde à largeur de télépho | Avis de conflit société : valeur longue sans espace qui déborde (`overflow-wrap`) | S |
| #1016 | P4 | 2026-11-11 | Réconciliation : la colonne Montant colle à la colonne Contrepartie, en-tête et cellu | Réconciliation : marge entre « Montant » et « Contrepartie » | S |
| #1096 | P4 | 2026-11-13 | Fiche facture validée : à 1400 px, la barre d'actions déborde et coupe le dernier bou | Fiche facture à 1400 px : la barre d'actions coupe le dernier bouton (retour à la ligne) | S |
| #1153 | P4 | 2026-11-14 | Réconciliation : les titres de colonnes « MontantContrepartie » et « Candidate » sont | Libellés de colonnes « MontantContrepartie » et « Candidate » | S |
| #1154 | P4 | 2026-11-14 | Journal d'audit : la colonne « N° » est collée à la colonne « Détails » | Journal d'audit : marge entre « N° » et « Détails » | S |
| #1184 | P4 | 2026-11-14 | Champ de compte : le bouton × chevauche la fin d'un libellé long | Champ de compte : le bouton × chevauche un libellé long | S |

**10 issues.** **Gates** : gate frontend ; **recette visuelle obligatoire** (captures avant/après de chaque écran touché, thème clair, base de démonstration) ; E2E complet à la clôture ; contraste : contrôle axe des specs existantes.

### Story 58-6 — Tests frontend de la réconciliation (35-4, 35-7)

**Module** : `frontend/src/lib/features/reconciliation/*.test.ts`, `journal-entries/AccountAutocomplete*`, `frontend/tests/e2e/reconciliation-*.spec.ts`.

| # | Prio | Créée | Titre | Correctif attendu | Coût |
|---|---|---|---|---|---|
| #1078 | P3 | 2026-11-13 | E2E 35-7 (AC7) : l'assertion sur le corps du POST passe aussi sur le code d'avant ; s | E2E 35-7 : l'assertion sur le corps du POST doit rougir sur l'ancien code | S |
| #1079 | P3 | 2026-11-13 | Vitest 35-7 (AC3, Écriture manuelle) : l'assertion « champ vide » passe sans l'effet  | Vitest : « champ vide » doit dépendre du `$effect` de `keepOfferedCounterparty` | S |
| #1080 | P4 | 2026-11-13 | Vitest 35-7 : les tests « comptes restés proposés » attendent un état déjà vrai avant | Vitest : « comptes restés proposés » attendent un état déjà vrai avant la résolution | S |
| #1082 | P4 | 2026-11-13 | E2E 35-7 (AC7) : l'option est choisie par dispatchEvent('mousedown'), non par un vrai | E2E : choisir l'option par un vrai clic, non `dispatchEvent('mousedown')` | S |
| #1083 | P4 | 2026-11-13 | Vitest : Éclater à trois lignes dont deux sur le compte de la banque n'est pas testé | Vitest : Éclater à trois lignes dont deux sur le compte de la banque | S |
| #1084 | P4 | 2026-11-13 | Champ de compte strict : un numéro complet tapé puis Tab est jeté, sans test qui fixe | Test qui fixe le comportement de « numéro complet puis Tab » | S |
| #1088 | P4 | 2026-11-13 | Réconciliation : la garde « if (!current) return; » de confirmGesture n'est plus exer | Retirer ou exercer la garde `if (!current) return;` de `confirmGesture` (l. 520) | S |
| #1090 | P3 | 2026-11-13 | E2E 35-4 : cliquerHors réessaie le clic jusqu'à la fermeture, ce qui masquerait un pr | E2E 35-4 : `cliquerHors` ne doit pas réessayer le clic jusqu'à la fermeture | S |
| #1091 | P4 | 2026-11-13 | E2E 35-4 : toContainText('1') accepte « 10 » ou « 11 » comme nombre de transactions c | E2E 35-4 : `toContainText('1')` accepte « 10 » ; asserter le nombre exact | S |
| #1093 | P4 | 2026-11-13 | Vitest 35-4 : bornes du nombre de cochées non testées (trois cochées dont une signalé | Vitest : bornes du nombre de cochées (trois dont une signalée ; toutes signalées) | S |

**10 issues.** **Gates** : vitest ciblé par fichier, puis E2E ciblés (`reconciliation-already-booked-reject`, `reconciliation-split`, `reconciliation-manual-entry`), E2E complet à la clôture ; aucun code de production attendu (hors garde #1088).
