# Glossaire de traduction — fr-CH → de-CH / it-CH / en-CH

*Établi le 2026-08-19 au kickoff de l'**Epic 23 « Dette i18n »** (issues [#316] et [#283]).*

Ce glossaire existe parce que l'epic 23 fait entrer **352 clés** au catalogue et écrit
**1056 messages** en trois langues — dont **20 clés livrées** par la story 23-1b (soit 60 messages : le décompte porte sur les **trois langues cibles**, les libellés `fr-CH` étant la source). Sans terminologie figée d'avance, trois cents décisions
lexicales se prennent une par une, au fil des stories de rollout, et le produit finit par
dire deux mots différents pour la même chose selon l'écran.

⚠️ **Les équivalences de la partie A ne sont pas des propositions : elles sont RELEVÉES
dans les catalogues existants**, sur les **1216 clés alignées sur les quatre locales au kickoff de l'epic** (2026-08-19 ; le pilote en a depuis ajouté 20).
La colonne « précédent » nomme la clé où l'équivalence est attestée.** Les changer, c'est
désaligner le nouveau du déjà-livré — ce que cet epic vient précisément corriger.

⚠️ **AUCUNE relecture par un locuteur natif n'est prévue, et c'est un choix.** Arbitrage de Guy,
2026-08-20 : *« le jour où un germanophone ou un italophone utilisera Kesh, il annoncera les erreurs
de traduction et nous prendrons les mesures à ce moment-là »*. Le coût d'une relecture
professionnelle est sans commune mesure avec celui d'une correction signalée, tant que le logiciel
n'a pas d'utilisateur dans ces langues.

**Ce que cette décision impose en retour, et qui n'est pas facultatif** : puisque rien ne rattrape
une erreur au moment où elle est écrite, **le relevé devient la seule discipline qui limite la
dérive**. Un terme déjà présent au catalogue se reprend tel quel ; un terme neuf se cherche d'abord
ailleurs dans les catalogues avant d'être proposé. ⚠️ **Le relevé n'est pas infaillible** — il
*propage* une erreur dont la première occurrence était fausse, et deux passes de revue en ont
attrapé de cette nature (`Aufwandkonto` pour `Aufwandskonto`, `Nr. Lieferantenrechnung` qui juxtapose
deux substantifs sans composition). Mais il borne le risque là où l'invention le laisse ouvert.

⚠️ **En cas de doute sur une langue cible, suivre le FRANÇAIS.** Arbitrage de Guy, 2026-08-19,
rendu sur `fattura fornitore` contre `fattura fornitori` : les deux étaient de l'italien correct, donc
rien ne permettait de trancher sans parler la langue. Le français, lui, est vérifiable — et il tranche
(`Facture fournisseur` au singulier, `Factures fournisseurs` au pluriel). **La ligne du glossaire
était bancale** : lemme français au singulier, italien au pluriel, une forme hybride correspondant à
ni l'un ni l'autre. Les catalogues, eux, étaient déjà justes.

⚠️ **Les termes sont donnés en forme de LEMME — minuscule et singulier —, les catalogues portant des
libellés d'interface.** `facture` relève `Factures`, `avoir` relève `Avoir`, `rappel` relève
`Rappels` : la mise au singulier et en minuscule est **uniforme** et ne constitue pas un écart. Ce
qui doit correspondre, c'est **le mot**, pas sa casse ni son nombre.

⚠️ **Cette convention a dû être écrite après coup, et elle a coûté deux erreurs symétriques.** La
passe 4 a marqué `personne de contact` comme « dérivé » parce que la clé citée porte le pluriel —
**sur-correction** : c'est la règle du document, pas une exception, et trente autres lignes font de
même sans mention. La passe 5 a ensuite rapporté la minuscule de `localité` et `prénom` comme un
écart — **même erreur en sens inverse**, sur deux lignes, après en avoir validé trente identiques.
Une convention tacite se fait donc prendre pour un défaut par qui la relit, et un défaut réel devient
indiscernable d'elle. C'est précisément ce que la ligne `field-city` avait révélé : là, l'écart était
**un autre mot** (`city` contre `Town/city`), et c'est ce genre d'écart, et lui seul, qui compte.

*(Revue de code 23-1b, passes 4 et 5.)*

La partie B, elle, appelle un arbitrage : **ces termes n'ont AUCUN précédent** dans les
catalogues. Ce qui y est écrit est une proposition, pas un relevé.

---

## Registre — mesuré, pas supposé

| Locale | Registre | Mesure |
|---|---|---|
| `fr-CH` | vouvoiement | 44 messages avec « vous / votre » |
| `de-CH` | **Sie-Form** | 115 **messages** avec « Sie » (117 *lignes* — les multi-lignes comptent double) |
| `it-CH` | **2ᵉ personne du singulier** (« Configura », « Scegli ») | 31 impératifs au singulier contre **11 messages** au registre de courtoisie (2 « Aggiungete » + 10 lignes en `vostro`) |
| `en-CH` | impératif neutre, sans pronom | — |

⚠️ **L'italien tutoie et l'allemand vouvoie.** Ce n'est pas une incohérence à corriger :
c'est l'usage courant des interfaces dans ces deux langues.

⚠️ **En revanche l'italien N'EST PAS homogène, et la première rédaction de ce § le sous-estimait.**
Elle annonçait « **unique** occurrence » du pluriel de courtoisie, à savoir
`accounts-create-description` (« Aggiungete un conto al piano dei conti. »). Recompté en passe 1 de
`validate` de la story 23-1, puis **re-recompté en passe 3** : **2 occurrences de « Aggiungete »** —
la seconde est `homepage-bank-empty-guided`, qui cumule l'impératif ET le possessif de politesse
(« Aggiungete **il vostro** conto ») — et **10 lignes** employant `vostro`/`vostra`/`vostri`/`vostre`,
soit **11 messages distincts** au registre de courtoisie.
⚠️ *La ligne du tableau ci-dessus a porté « 31 contre 1 » pendant deux passes APRÈS que cet encadré
l'eut réfutée : la passe 1 avait corrigé la prose et laissé le tableau, à dix lignes de distance.
Corrigé en passe 3.* *Une
affirmation portant le titre « mesuré, pas supposé » et qui ne l'était qu'à moitié : qui corrigerait
sur la foi de la version précédente croirait le défaut borné à une ligne.* L'alignement se fait au
passage des rollouts qui touchent ces domaines, pas ici.

---

## A. Termes attestés — équivalences relevées dans le catalogue

| fr-CH | de-CH | it-CH | en-CH | précédent |
|---|---|---|---|---|
| facture | Rechnung | fattura | invoice | `Factures` / `Rechnungen` |
| facture fournisseur | Lieferantenrechnung | **fattura fornitore** *(pluriel : `fatture fornitori`)* | supplier invoice | `supplier-invoices-detail-title` (sing.) / `supplier-invoices-title` (plur.) |
| avoir | **Gutschrift** | **nota di credito** | credit note | `Avoir` / `N° d'avoir` |
| brouillon | Entwurf | bozza | draft | `Brouillon` |
| ouverte (facture) | offen | aperta | open | `Factures ouvertes` |
| payée / impayée | bezahlt / unbezahlt | pagata / non pagata | paid / unpaid | `Payées` / `Impayées` |
| **annulé(e)** | **storniert** | **annullato / annullata** | **cancelled** | `invoice-status-cancelled` (story 23-3b) — ⚠️ la clé atteste `Annulée` / `Storniert` / `Annullata` / `Cancelled` : **`Annulé` (masc.) et `annullato` sont des accords DÉRIVÉS**, relevé en passe 3. ⚠️ **L'ACCORD suit la langue, pas le français** : un *lot* est masculin (`annullato`), une *nota di credito* féminine (`annullata`) ; l'allemand est invariable |
| **créé(e)** | **erstellt** | **creato / creata** | **created** | `imported-supplier-invoices-completed` (story 23-3b, arbitrage de Guy du 2026-08-20) — ⚠️ **relevé au participe**, contrairement à `confirmé` et `émis` : la clé dit « Facture **créée**. / Rechnung **erstellt**. / Fattura **creata**. / Invoice **created**. » ; seul le masculin `creato` est un accord dérivé. ⚠️ **Le français a CHANGÉ** : le statut de lot disait « Généré ». Le verbe *créer* est désormais uniforme dans le domaine — `Créer une facture`, `Facture créée.`, un lot `Créé` — soit un mot de moins à retenir pour l'utilisateur |
| **confirmé(e)** | **bestätigt** | **confermato / confermata** | **confirmed** | ⚠️ **DÉRIVÉ, non relevé** (story 23-3b, passe 1 de revue). `demo-reset-confirm-ok` porte l'**infinitif** — `Confirmer` / `Bestätigen` / `Conferma` / `Confirm` —, pas le participe. **Aucune forme participiale n'existait dans les catalogues avant cette story** : ces quatre valeurs sont une transformation grammaticale, pas un relevé. La première rédaction citait cette clé comme *attestante*, ce qui était faux |
| **émis(e)** | **ausgestellt** | **emesso / emessa** | **issued** | `credit-note-revenue-account-archived` (story 23-3b) — ⚠️ **relevé pour l'ALLEMAND SEUL** : la clé porte bien `ausgestellt` en participe (tournure passive), mais l'italien et l'anglais y sont à l'**infinitif** (`emettere`, `issue`), de même que le français (`émettre`). **`emesso`/`emessa`/`issued`/`Émis` sont donc DÉRIVÉS.** ⚠️ **Ne pas confondre avec *validé***, qui désigne l'acte d'immuabilité d'une facture : émettre un avoir et valider une facture sont deux actes distincts, dont un seul est irréversible |
| **relevé (bancaire)** | **Kontoauszug** | **estratto conto** | **statement** | `homepage-bank-empty-guided` (story 23-3b). Relevé en cours de route, hors des trois statuts qu'AC6 exigeait : `nav-bank-import` l'emploie et le fige, donc il monte |
| en retard | **überfällig** | in ritardo | overdue | `En retard` |
| échéance | **Fälligkeit** | scadenza | due date | `Échéance` |
| conditions de paiement | Zahlungsbedingungen | condizioni di pagamento | payment terms | `Conditions de paiement` |
| écriture (comptable) | **Buchung** / Buchungssatz | registrazione / scrittura contabile | entry / journal entry | `Dernières écritures` |
| compte | Konto | conto | account | `Comptes bancaires` |
| compte bancaire | Bankkonto | conto bancario | bank account | `Comptes bancaires` |
| solde | Saldo | saldo | balance | `Soldes de départ` / `Anfangssaldi` |
| montant | **Betrag** | **importo** | amount | `Montant trop élevé` |
| montant HT | Betrag exkl. MWST | importo IVA esclusa | amount excl. VAT | `Montant HT` |
| **montant TTC** | **Betrag inkl. MWST** | **importo IVA inclusa** | **amount incl. VAT** | `supplier-invoices-field-expected-amount` (story 23-3, passe 4). ⚠️ **Le pendant exact de *montant HT*, et il n'était protégé par rien** alors que celui-ci l'était — asymétrie relevée en passe 4. `credit-notes-col-total` et `-col-line-total`, encore à la dette, porteront la forme opposée |
| **compte de charge** | **Aufwandskonto** | **conto di costo** | **expense account** | `vat-purchase-charge-account` (story 23-3, passe 4). ⚠️ **Le SEUL terme du domaine dont on ait la PREUVE qu'il dérive** : la passe 1 a dû corriger `Aufwandkonto` et `conto costi`, écrits trois fois chacun. ⚠️ **L'anglais s'en tire par coïncidence** (terme monolithique) — un contrôle mené sur lui seul ne l'aurait pas vu |
| exercice (comptable) | **Geschäftsjahr** | esercizio contabile | fiscal year | `Exercices comptables` |
| résultat de l'exercice | Jahresergebnis | risultato dell'esercizio | current year result | `Résultat de l'exercice` |
| bilan | Bilanz | bilancio | balance sheet | `Bilan` |
| compte de résultat | **Erfolgsrechnung** | **conto economico** | income statement | `Compte de résultat` |
| journal | Journal | **giornale** | journal | `Journal` |
| TVA | **MWST** | IVA | VAT | `Taux TVA` / `MWST-Satz` |
| taux (TVA) | Satz | aliquota | rate | `Taux TVA` |
| décompte TVA | MWST-Abrechnung | rendiconto IVA | VAT settlement | `Décompte TVA` |
| TVA due | geschuldete MWST | IVA dovuta | VAT payable | `TVA due` |
| produit (compte de) | **Ertrag** | **ricavo** | revenue | `Produit` / `Produit par défaut` |
| créances clients | Forderungen aus Lieferungen und Leistungen | crediti verso clienti | trade receivables | `Créances clients` |
| dettes fournisseurs | Verbindlichkeiten aus Lieferungen und Leistungen | debiti verso fornitori | trade payables | `Dettes fournisseurs` |
| client | Kunde | cliente | client | `Client` |
| numéro de client | Kundennummer | numero cliente | client number | `Numéro de client` |
| contact | Kontakt | contatto | contact | `Nouveau contact` |
| **localité** | **Ort** | **località** | **Town/city** | `field-city` (story 23-1b) |
| **prénom** | **Vorname** | **nome** | **first name** | `field-first-name` (story 23-1b) |
| **personne de contact** | **Kontaktperson** | **persona di contatto** | **contact person** | `contact-persons-title` — au pluriel dans les quatre catalogues (`Personnes de contact`, `Kontaktpersonen`, `Persone di contatto`, `Contact persons`), mis au singulier comme partout ailleurs ici (story 23-1b) |
| **immuable** | **unveränderlich** | **immutabile** | **immutable** | `invoice-validate-confirm-body` (story 23-2) |
| **validité** | **Gültigkeit** | **validità** | **validity** | `vat-rates-subtitle` (story 23-2) |
| **bascule** (*changement de taux*) | **Umstellungsdatum** | **data di cambiamento** | **changeover date** | `vat-rates-field-switch-date` (story 23-2). ⚠️ **HOMONYME** — rien à voir avec la bascule-interrupteur de la partie B |
| **virement** | **Überweisung** (`Banküberweisung` si le mode de paiement) | **bonifico** | **bank transfer** | `supplier-invoices-pay-transfer` (story 23-3). Usage bancaire suisse standard |
| **image** | **Bild** | **immagine** | **image** | `supplier-invoices-scan-failed` (story 23-3). Scan de QR-facture |
| **compléter** | **vervollständigen** | **completare** | **complete** | `imported-supplier-invoices-complete` (story 23-3) |
| **écarter / écartée** (une pièce) | **verwerfen / verworfen** | **scartare / scartata** | **discard / discarded** | `imported-supplier-invoices-discard` (story 23-3). S'oppose à « compléter » dans la file d'import. ⚠️ En italien, **ne pas employer `scarto` pour un ÉCART de montant** sur le même écran — dire `differenza` |
| **justificatif** | **Beleg** | **documento giustificativo** | **supporting document** | `imported-supplier-invoices-view-doc` (story 23-3). `Beleg` est le terme du CO art. 957a ; « Buchungsbeleg » si le contexte est l'écriture |
| **QR-facture** *(fém.)* | **QR-Rechnung** | **fattura QR** | **QR-bill** | `supplier-invoices-scan` (story 23-3). ⚠️ Terminologie **officielle SIX**. ⚠️ **FÉMININ en français** — « une QR-facture », « QR-facture lue » : quatre libellés l'accordaient au masculin, et les trois cibles avaient bon |
| **restauré** | **wiederhergestellt** | **ripristinato** | **restored** | `imported-supplier-invoices-doc-gone` (story 23-3). Le justificatif qu'une restauration de sauvegarde n'a pas ramené |
| **règlement** (*d'une facture*) | **Zahlung** (`Zahlungsdatum`) | **pagamento** | **payment** | `supplier-invoices-pay-date` (story 23-3). ⚠️ **Le français seul distingue** *règlement* de *paiement* ; les trois cibles n'ont qu'un mot et disent **paiement**. Ne PAS écrire `Begleichung` / `settlement` — deux mots pour un concept déjà couvert |
| carnet d'adresses | **Kontakte** | **contatti** | contacts | `Carnet d'adresses` |
| adresse | Adresse | indirizzo | address | `Adresse` |
| rappel / relance | **Mahnung** | **sollecito** | reminder | `Rappels` / `Mahnungen` |
| paiement | Zahlung | pagamento | payment | `Paiements fournisseurs` |
| réconciliation | **Abstimmung** (manuelle : Abgleich) | riconciliazione | reconciliation | `Réconciliation` |
| import bancaire | Bankimport | importazione bancaria | bank import | `Import bancaire CAMT.053` |
| sauvegarde | Sicherung | backup | backup | `Sauvegarde complète` |
| archiver / archivé | archivieren / archiviert | archiviare / archiviato | archive / archived | `Archiver` / `Archivé` |
| invalide | ungültig | non valido | invalid | `Valeur invalide` |
| référence | Referenz | riferimento | reference | `Référence` |
| utilisateur | Benutzer | utente | user | `Utilisateurs` |
| rôle | Rolle | ruolo | role | `Rôle` |
| paramètres | **Einstellungen** | **impostazioni** | settings | `Paramètres` |
| rapport | Bericht | rapporto | report | `Rapports comptables` |
| banque | Bank | banca | bank | `Banque` |
| devise | Währung | valuta | currency | `Devise non supportée v0.1.` |
| projet | Projekt | progetto | project | `supplier-invoices-detail-project` |
| **projet analytique** | **Projekt** | **progetto** | **project** | arbitrage de Guy, 2026-08-19 — cf. la note ci-dessous |
| dépenses | Ausgaben | spese | expenses | `reports-filename-project-expenses` ⚠️ *slug de nom de fichier, pas un libellé* |

⚠️ **« Projet analytique » se rend simplement par `Projekt` / `progetto` / `project`** — arbitrage
de Guy du 2026-08-19, et il mérite sa justification parce que la proposition initiale de ce
glossaire était **fausse**.

Elle suggérait `Kostenstelle` / `centro di costo` / *cost center*. Or une **Kostenstelle** répond à
« **où** le coût est-il né ? » — un service, un atelier, une unité permanente de l'organisation.
Ce que Kesh appelle projet répond à « **pour quoi ?** » : la table `projects`
(`20260702000001_projects_analytics.sql`) porte un code, un nom, deux niveaux de hiérarchie et des
dates de début et de fin. C'est un objet **temporaire porteur de coûts** — un `Kostenträger` en
allemand, une `commessa` en italien. **Le terme proposé désignait le mauvais concept.**

Le choix retenu n'est pourtant ni l'un ni l'autre : **on ne traduit pas « analytique »**. Motifs —
`Projekt` est **déjà attesté** au catalogue (`supplier-invoices-detail-project`, et avant lui le slug `reports-filename-project-expenses`) ; l'utilisateur lit ce que le
logiciel fait réellement ; et une étiquette de champ n'est pas l'endroit où faire de la terminologie
comptable. `Kostenträger` et `commessa` ont leur place dans le **manuel**, pas dans l'interface.

Concerne quatre libellés, tous portés par les rollouts 23-3 et 23-5 : « Projet analytique »,
« Projet analytique (optionnel) », « Projet analytique par défaut », « Projets analytiques ».

---

## B. Termes SANS précédent — arbitrage requis

⚠️ **Aucune de ces équivalences n'est attestée dans le catalogue.** Elles sont proposées,
avec leur motif. Une fois tranchées, elles remontent en partie A et deviennent
contraignantes pour tout le rollout.

| fr-CH | de-CH proposé | it-CH proposé | en-CH proposé | motif / réserve |
|---|---|---|---|---|
| personne physique / morale | natürliche / juristische Person | persona fisica / giuridica | individual / legal entity | |
| bascule (**interrupteur**) | Umschalter | interruttore | toggle | élément d'interface. ⚠️ **NE PAS employer pour « date de bascule »**, qui est un changement de taux et vit en partie A |

---

✅ **Treize de ces termes ONT ÉTÉ tranchés et promus en partie A**, chacun portant désormais la clé
qui l'atteste : **localité**, **prénom** et **personne de contact** par la story 23-1b ; **immuable**
et **validité** par la 23-2 ; **QR-facture**, **justificatif**, **écarter/écartée**, **compléter**,
**image** et **virement** par la 23-3, puis **restauré** et **règlement** par sa passe 2 de revue.

⚠️ **`bascule` reste en partie B, et sa ligne y porte un avertissement** — la 23-2 a révélé qu'il
s'agit de **deux termes sous un seul mot**. L'entrée de partie B est l'**interrupteur** d'interface ;
« date de bascule », qui désigne le **changement d'un taux de TVA**, est monté en partie A avec une
traduction sans rapport. Traduire la seconde d'après la première aurait donné un contresens plein —
et c'est le glossaire lui-même qui aurait induit en erreur, ce qui est pire qu'un glossaire muet.

**Deux** termes restent ouverts. *(La partie B comptait 16 entrées, 15 après l'arbitrage sur
« analytique », 12 après la 23-1b, 10 après la 23-2, 4 après la 23-3 et **2** après sa passe 2. La
partie A compte **70** entrées. Les deux nombres sont **recomptés depuis les tableaux**, comme
l'exige la § « Recompter ses propres comptes rendus ».)*

⚠️ **65 → 70 à la story 23-3b**, et le périmètre de la promotion se déclare avec le nombre :
**trois** entrées l'étaient par son AC6 — `créé`, `confirmé`, `émis`, les statuts que le relevé
ne couvrait pas et que Guy a tranchés le 2026-08-20 — et **deux** ont été relevées en chemin,
`annulé(e)` et `relevé (bancaire)`. Ces deux-là illustrent le défaut que le paragraphe suivant
décrit : elles étaient **attestées au catalogue et absentes de la partie A**, donc libres de
dériver. `annulé(e)` est le cas net — la spec de cette story le donnait pour « ✅ attesté » sans
que personne ne remarque qu'il n'était protégé par rien.

⚠️ **« Relevé » et « dérivé » ne sont PAS la même chose, et la colonne de droite doit le dire.**
Un terme est **relevé** quand la clé citée porte *exactement* la forme écrite. Il est **dérivé**
quand il a fallu une transformation — accord de genre, ou passage de l'infinitif au participe.
La distinction n'est pas cosmétique : le préambule pose le relevé comme **la seule discipline qui
borne la dérive**, faute de relecture par un locuteur natif. Un terme dérivé présenté comme relevé
retire ce garde-fou tout en donnant l'apparence de l'avoir appliqué.

⚠️ **Le défaut a été commis puis corrigé DANS CETTE MÊME STORY** : `confirmé` et `émis` citaient
des clés qui portent l'**infinitif**, et les Completion Notes affirmaient « aucun terme n'a été
inventé ». Relevé en passe 1 de revue, vérifié au grep — aucune forme participiale n'existait dans
les catalogues avant. C'est le troisième précédent de cette famille sur ce dossier, après le
`city`/`Town/city` de la 23-1b et le « personne de contact » de sa passe 4 : **le glossaire se
fait réfuter par les clés qu'il cite comme preuves**, et seule une vérification au catalogue le
détecte.

⚠️ **Les six codes d'échec de `failedItemLabel` ne montent PAS en partie A** — décision de Guy :
ce sont des libellés d'erreur, pas des termes métier. Le glossaire porte le vocabulaire que
l'utilisateur lit et reconnaît d'un écran à l'autre ; y verser des messages techniques le
diluerait sans rien protéger.

⚠️ **Les deux dernières entrées de la partie A n'y sont PAS montées depuis la partie B** — `montant TTC`
et `compte de charge` n'y ont jamais figuré : ils étaient « attestés au catalogue », ce que la spec de
la 23-3 tenait pour suffisant. **Ça ne l'est pas.** La règle d'immuabilité ne couvre que la partie A,
si bien qu'un terme attesté mais non promu dérive sans que rien ne rougisse — et `compte de charge`
en est la **preuve** : la passe 1 a dû corriger `Aufwandkonto` et `conto costi`, trois fois chacun.
D'où le critère, désormais : **un terme que le rollout emploie et fige va en partie A**, qu'il vienne
de la partie B ou du catalogue. *(Passe 4 de la 23-3 ; la partie A passe de 63 à 65.)*

⚠️ **La 23-3 avait d'abord OUBLIÉ cette promotion, en cochant sa tâche.** Elle a employé et figé
**huit** de ces termes dans 460 lignes de catalogue **sans les faire monter en partie A** — or la
règle d'immuabilité ne protège **que** la partie A. Un rollout suivant aurait pu écrire
`Buchungsbeleg` ou `Quittung` pour « justificatif » sans que rien ne rougisse : le mécanisme
anti-dérive que cet epic existe pour poser, contourné par la story censée s'en servir.

⚠️ **Et le correctif de la passe 1 n'en a promu que SIX sur huit** — `restauré` et `règlement`
étaient employés eux aussi, sur trois clés, et sont restés en partie B une passe de plus. Le défaut
d'un patch de remédiation est du même genre que celui qu'il corrige : **la question n'était pas
« ai-je promu les termes que j'ai listés ? » mais « quels termes de partie B ce catalogue
emploie-t-il ? »** — c'est-à-dire un grep du symptôme, et non du site. Trouvé en passe 2.

## Comment s'en servir

1. **Avant d'écrire une traduction**, chercher le terme ici. S'il y est, l'employer — sans
   variante « qui sonne mieux ».
2. **S'il n'y est pas et qu'il est récurrent**, l'ajouter en partie B plutôt que de trancher
   dans une story : le prochain rollout retrouvera la décision au lieu de la reprendre.
3. **Un terme de la partie A ne se change pas dans une story de rollout.** Il se change dans
   une story qui met AUSSI à jour toutes ses occurrences déjà livrées — sinon le produit
   dit deux mots pour une chose, ce que ce fichier existe pour empêcher.

[#316]: https://github.com/guycorbaz/kesh/issues/316
[#283]: https://github.com/guycorbaz/kesh/issues/283
