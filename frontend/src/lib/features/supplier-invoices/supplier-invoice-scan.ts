/**
 * Scan QR-facture (Story 12.4, #191) — décodage côté navigateur.
 *
 * DC1 : upload d'une image → décodage local via `jsQR` sur un `<canvas>`, PAS la
 * caméra (`getUserMedia` est secure-context-only → KO en HTTP LAN). `createImageBitmap`
 * et le canvas 2D fonctionnent en contexte non sécurisé (fichier local, même origine).
 */

import jsQR from 'jsqr';
import type { ScanQrResponse } from './supplier-invoices.types';

/**
 * Décode le premier QR-code trouvé dans un fichier image. Retourne le texte brut
 * (payload SPC pour une QR-facture) ou `null` si aucun QR n'est détecté.
 * Ne lève pas sur une image sans QR ; lève uniquement si le fichier n'est pas
 * une image décodable (géré par l'appelant).
 */
export async function decodeQrFromImageFile(file: File): Promise<string | null> {
	const bitmap = await createImageBitmap(file);
	try {
		const canvas = document.createElement('canvas');
		canvas.width = bitmap.width;
		canvas.height = bitmap.height;
		const ctx = canvas.getContext('2d');
		if (!ctx) return null;
		ctx.drawImage(bitmap, 0, 0);
		const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
		const result = jsQR(imageData.data, imageData.width, imageData.height);
		return result?.data ?? null;
	} finally {
		bitmap.close();
	}
}

/** Valeurs de pré-remplissage dérivées d'une réponse `scan-qr` (chaînes prêtes à binder). */
export interface ScanPrefill {
	creditorIban: string;
	creditorQrIban: string;
	paymentReference: string;
	expectedAmount: string;
	/** Devise du QR (`CHF`/`EUR`) — affichée avec le montant pour lever l'ambiguïté. */
	currency: string;
	creditorName: string;
}

/**
 * Mappe la réponse backend vers les champs du formulaire (pur, testable).
 * Le backend garantit qu'au plus l'un de `creditorIban` / `creditorQrIban` est
 * renseigné ; les `null` deviennent des chaînes vides (champs contrôlés).
 */
export function scanToPrefill(scan: ScanQrResponse): ScanPrefill {
	return {
		creditorIban: scan.creditorIban ?? '',
		creditorQrIban: scan.creditorQrIban ?? '',
		paymentReference: scan.paymentReference ?? '',
		expectedAmount: scan.expectedPaymentAmount ?? '',
		currency: scan.currency ?? '',
		creditorName: scan.creditorName ?? '',
	};
}
