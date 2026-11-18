# Story 43-3 — registre des choix (développement, 2026-11-17)

Registre de la story 43-3 (clé `e433`). Les décisions de l'épique (C-43-14 à C-43-26), la fiche 43-3 et la règle de la
différence (`kesh-gate-logs/e43/decision-difference.md`) font foi ; ce registre consigne les choix tranchés par l'agent de
développement. Chaque choix est réversible sans migration.

**C-e433-1 — Annulation de la recherche par `AbortController`, sans toucher `api-client`.** La fiche (D4) veut la requête
précédente « annulée par `AbortController` ». `apiClient.get` n'accepte aucun signal (`api-client.ts`, doc-comment de
`fetchWithTimeout` : le signal d'un appelant serait écrasé), et y en faire passer un traiterait l'abandon comme un délai
dépassé : `toFetchApiError` en ferait un `TIMEOUT`, `fetchWithRetry` rejouerait la requête et `apiHealth.setDegraded()`
afficherait le bandeau « serveur dégradé » à chaque frappe. Retenu : `modal-guards.ts` tient un `AbortController` par
requête, l'avorte quand une requête plus récente part ou que la modale se ferme, et toute réponse dont le signal est avorté
ou la génération dépassée est ignorée. La requête réseau va à son terme, son résultat est jeté. Autre option : étendre
`CallOptions` d'un `signal` et distinguer l'abandon du délai dans `fetchWithRetry` — module partagé par tout l'écran,
hors du périmètre de la story ; une issue est demandée au rapport. Réversible : le signal est déjà porté par l'appel.

**C-e433-2 — Réglages de facturation illisibles : l'écran ne bloque pas sur l'arrondi.** La modale lit
`GET /company/invoice-settings` à l'ouverture (comptes désignés). Si la lecture échoue, elle ne sait pas si un compte
d'arrondi est désigné : elle ne préremplit rien et ne refuse pas l'envoi pour `r ≠ 0` ; le serveur tranche
(`ROUNDING_ACCOUNT_NOT_CONFIGURED`, affiché avec son libellé). Autre option : traiter l'échec comme « aucun compte
désigné » — la phrase « Aucun compte de différences d'arrondi n'est désigné » serait alors fausse.

**C-e433-3 — Le compte de l'écart effectif est dérivé, non stocké.** Le compte envoyé est
`choisiParUtilisateur ? choixUtilisateur : préremplissage(sens)`, calculé à chaque rendu (`effectiveDifferenceAccount`,
`group-settlement.ts`). Le préremplissage suit donc le signe sans effet de bord ; le drapeau survit au changement de signe
et à la disparition du sélecteur ; vider le champ lève le drapeau et rend le préremplissage. Un choix sorti de la liste
(`keepOfferedCounterparty`) retombe à vide, comme le #1000.

**C-e433-4 — Relecture des montants par la recherche, une requête par facture imputée.** Sur
`difference_account_required` ou `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE`, la fiche veut les montants des factures
imputées relus. Aucune route ne rend une candidate par identifiant ; la recherche, elle, rend `amountDueRaw` sur les mêmes
règles que le `POST`. La modale cherche chaque facture imputée par son numéro (`q` = numéro, numéro interne à défaut pour
un fournisseur), garde la ligne de même identifiant et met son reste dû à jour ; une facture que la recherche ne rend plus
est retirée, avec la phrase « Facture … : n'est plus réglable ». Autre option : `GET /invoices/{id}` — son `amountDue`
n'est pas garanti au brut, ni défini pour une facture fournisseur.

**C-e433-5 — Montants en `bigint` de 1/10 000.** « Aucun `Number` sur un montant » (D3) : les montants se lisent en
`bigint` (unités de 1/10 000) par `parseUnits`, qui refuse plus de quatre décimales et toute forme autre que
`-?\d+(\.\d{1,4})?`. Un `bigint` n'a pas de borne à 2⁵³.

**C-e433-6 — Gardes en fabriques, non en classes.** `modal-guards.ts` expose `createSessionGuard` et
`createLatestRequest` (fermetures). La garde `i18n-libelle-en-dur.test.ts` refuse toute classe non admise
dans `src/` (les méthodes de classe échappent à son relevé des libellés en dur) ; une première rédaction en
classes l'a fait rougir. Autre option : admettre les deux classes nommément (`CLASSES_ADMISES`), comme
`SecretUnique` — une exception de plus à une garde qui en compte une.

**C-e433-7 — Côté fournisseur en commits à part (D6, C-43-18).** Le commit `fefaed04f` (code) et le
commit `81e4b4a2b` (E2E) portent tout ce qui ne sert qu'au débit : la variante `supplier_invoice_group` du
type, la branche du corps, les libellés « Débit supérieur / inférieur aux factures », la colonne
« Fournisseur », le texte d'annulation d'un groupe fournisseur, leurs quatre clés dans les quatre locales,
`canGroup` élargi au débit et leurs tests ; `sitesTotal` 3142 → 3146. La logique pure (`gapSense`,
`prefillAccount` pour les deux familles) reste dans le commit de base : elle ne sert à rien sans le bouton,
et ses tests n'appellent aucune API. Retirer les deux commits ramène le bouton au seul crédit.

**C-e433-8 — Cinquante factures au plus, dit à l'écran.** À 50 factures imputées, les boutons *Ajouter*
se grisent et une phrase le dit (`reconciliation-group-max-reached`) — la borne `too_many` de la 43-1a
n'est jamais atteinte depuis l'écran. Autre option : laisser le serveur refuser (`too_many`), libellé
existant.

**C-e433-9 — Une modale fermée par un refus prévient le parent par `onStale`.** Distinct d'`onSuccess` :
le parent relit sa liste sans compter d'opération réussie ni effacer le bilan du lot précédent.

**C-e433-10 — Le refus `ACCOUNT_IN_FOREIGN_CURRENCY` d'un groupe (Epic 37, apporté par la 43-2).** Sans
discriminant, la clé globale existante `error-account-in-foreign-currency` (elle nomme la devise) ; avec le discriminant
`group` et `details.accountId` égal au compte envoyé, une forme propre au compte de l'écart
(`reconciliation-failed-group-account-foreign-currency`, quatre locales), affichée sous le sélecteur. Le sélecteur
exclut déjà ces comptes (`counterpartyAccounts`, et `excludeForeignCurrency` de `AccountAutocomplete`, posé
explicitement) : le refus ne survient qu'en course (compte passé en devise entre-temps). 48 codes au test.

**C-e433-11 — Les états fugaces de l'E2E par une réponse interceptée.** Chargement, `hasMore`, `direction: null`,
refus du serveur par cause, réglages altérés et refus d'annulation par facture s'obtiennent dans
`reconciliation-group.spec.ts` en altérant **une** réponse (`page.route`, précédents : `invoices.spec.ts`,
`reminders.spec.ts`), sur un décor réel (factures, relevé, comptes). Les gestes nominaux, eux, traversent la frontière
HTTP sans interception. Autre option : fabriquer chaque état en base (compte passé en devise, avoir sur une facture
groupée, plus de 50 candidates) — plusieurs heures pour des captures. La recherche « sans résultat » est interceptée
aussi : la recherche de la 43-2 est une disjonction de jetons, et un terme fait de jetons uniques à l'exécution trouvait
encore le tiers du décor (`e2e-group-3.log`).

**C-e433-12 — Relecture des montants : une facture non retrouvée reste (revue P1, B-M2, E-L1 ; remplace la
seconde moitié de C-e433-4).** L'API n'offre aucune lecture d'une candidate par identifiant avec son reste dû
**brut** et ses conditions de réglabilité (date, lot) : `GET /invoices/{id}` rend un `amountDue` sans garantie de
brut, et rien n'existe pour une facture fournisseur. La relecture garde donc la recherche par numéro, mais une
facture qu'elle ne rend pas — numéro absent (`#id`), recherche trop large, erreur — n'est **jamais** retirée ni dite
« plus réglable » : elle reste, avec son montant d'avant, et une phrase dit qu'elle n'a pas pu être relue ; le
serveur tranche à l'enregistrement, et ses refus nommés (D5) la retirent s'il le faut. Autre option : relire par
`GET /invoices/{id}` pour les clients seulement — deux chemins pour une même grandeur, et un reste dû dont la forme
n'est pas celle du calcul.

*Revue P2 (L-1).* La phrase « non relue » ne s'efface plus avec les refus : elle reste tant que la facture est
imputée sans avoir été relue, dit que l'écart affiché peut être faux et comment la relire (la retirer puis l'ajouter
de nouveau : ajoutée depuis la recherche, elle porte un reste dû frais). Quand le serveur a demandé un compte de
l'écart (`difference_account_required`), le sélecteur s'affiche même si l'écart calculé à l'écran est nul : l'écran
ne reste plus dans l'impasse d'un refus qu'aucun geste ne lève.

*Pourquoi deux codes seulement.* La relecture ne part que sur `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE` et
`RECONCILIATION_GROUP_INVALID` / `difference_account_required` : ce sont les seuls refus du règlement groupé qui
portent sur l'écart, donc sur les montants. Les autres disent une facture (`RETIRER_SUR`, retirée et nommée), la
transaction (`FERMER_SUR`, fermeture), un compte ou une configuration — rien qu'une relecture des restes dus ne
change. Recherche d'un autre refus de montant (`grep -rhoE "RECONCILIATION_[A-Z_]+" crates/*/src`, 2026-11-18) :
seul `RECONCILIATION_AMOUNT_MISMATCH`, propre au règlement d'**une** facture fournisseur (`|tx| = TTC`), que le
groupe n'émet pas. Aucun code n'est ajouté.

*Revue P3 ciblée (L-A).* La demande du serveur (`serverWantsAccount`) ne s'impose que si une facture n'a pas pu être
relue : une relecture complète la lève (les montants de l'écran sont alors ceux du serveur), comme tout changement des
factures (ajout, retrait, facture retirée sur refus). Relue à écart nul, la fenêtre dit « Les restes dus ont été
relus : vérifiez l'écart, puis enregistrez de nouveau. » — ni sélecteur, ni compte exigé, jamais un refus muet.

*Revue P3 ciblée (L-B) — écart à la consigne, déclaré.* La consigne était de remettre le choix du compte à zéro
quand l'écart retombe à zéro. Elle contredit la fiche validée (D2 bis, « Choix gardé », P4 L5 : le choix survit
« si le sélecteur disparaît puis réapparaît ») et C-e433-3, que fige le test « choix gardé ». Retenu, dans le
périmètre de la fiche : le choix reste ; quand le serveur demande un compte alors que le champ en porte déjà un que
l'envoi n'a pas porté (écart nul à l'envoi), la phrase sous le sélecteur dit « Kesh demande un compte pour l'écart :
vérifiez le compte choisi, puis enregistrez de nouveau. » au lieu de « Choisissez le compte de l'écart ». Le cas du
compte prérempli (relecture qui fait apparaître un écart) prend la même phrase. Remettre le choix à zéro reste
possible, mais change une règle de la fiche : décision au Project Lead.

*Revue P3 ciblée (L-D) — rédaction laissée telle quelle, noté.* « Retirez-la puis ajoutez-la de nouveau » ne se suit
que si la recherche courante ramène la facture : `removeInvoice` relance la recherche avec la requête du champ, et
une facture trouvée par son numéro ne revient pas si l'utilisateur a changé la requête. Le texte reste vrai dans le
cas courant ; une autre rédaction (« relancez la recherche par son numéro ») ou la remise de la requête au retrait
sont à trancher avec Guy.

**C-e433-13 — Le « × » du compte de l'écart laisse le champ vide (revue P1, B-L2).** Vider le champ pose le choix
« aucun compte » (`selectAccount(null)` → `{ byUser: true, accountId: null }`) : le préremplissage ne revient pas,
le compte est demandé, et *Enregistrer* le dit. La fiche (D2 bis) disait « [le drapeau] se lève si l'utilisateur
vide le champ » : le geste n'avait alors aucun effet visible, le compte prérempli réapparaissant aussitôt. Le choix
se lève à la réouverture de la modale.

**C-e433-14 — Hors story, noté (revue P1, E-L10).** Un plafond stocké à quatre décimales par une écriture SQL
directe (5.0050) est rendu arrondi par l'API (`maxDifference` à deux décimales, 5.01) : l'écran admet un écart que
le `POST` refuse, et le refus revient à chaque clic. Angle mort de la 43-7 (le réglage par l'écran est au centime),
renvoyé à la 43-7 et à #1245.

**C-e433-15 — Fermeture pendant l'envoi (revue P1, E-L7).** Corrigé en partie : le toast de succès part même la
modale fermée. Reste : modale rouverte sur une autre transaction pendant l'envoi, le parent ne relit pas sa liste
(appeler `onSuccess` fermerait la modale rouverte) ; la ligne réglée reste jusqu'à la relecture suivante, et un
geste dessus finit en `NOT_PENDING` (toast et fermeture). Tracé par l'issue #1262 (revue P2, L-7).
