# 3. Arquitectura

Decidida con el dueño del equipo el 2026-11-14. El acceso a cada fuente de datos se probó en vivo: ver `research/01_api_tests.md`.

## Un producto, un mundo

- **Index (portada):** un globo vivo en Three.js al amanecer con imágenes reales de la NASA (Blue Marble, luces nocturnas de VIIRS). Al bajar, la cámara desciende de la órbita al valle. Al frente, un teléfono de vidrio esmerilado muestra la app en uso. Referencia aprobada: `design-directions/approved-index-hero.webp`.
- **App:** el mismo mundo continúa. El fondo pasa a ser el propio lote del agricultor visto desde arriba; la interfaz es vidrio esmerilado flotando encima.
- **Mismas funciones en celular y escritorio:** ubicación (GPS, punto en el mapa o contorno dibujado, búsqueda de lugar), hasta 10 fotos por mensaje (cámara o arrastrar y soltar), chat con el consultor, apartados de resultados, dashboard y exportación.

## Componentes

| Capa | Elección | Por qué |
|------|----------|---------|
| Frontend | TypeScript, HTML, CSS; Three.js para el globo; un mapa web para el lote | Elección del dueño; una sola app web adaptable (PWA instalable) para celular y escritorio |
| Backend | Rust | Preferencia permanente del dueño; rápido, un solo binario |
| Base de datos | MariaDB 11.4 LTS | Elección del dueño; código abierto (GPL); tipos geográficos para los contornos; buen soporte en Rust (`sqlx`) |
| Archivos | Disco del servidor o Google Cloud Storage | Fotos y PDF exportados quedan fuera de la base de datos |
| Consultor IA | Claude vía la API de Anthropic | Dos etapas: un analista arma un paquete fijo de evidencia con datos NASA y del catálogo; el consultor lo presenta y solo puede citar sus cifras |
| Hosting | Una máquina virtual e2-micro de Google Compute Engine (capa gratuita permanente) con la API en Rust, MariaDB y un servidor web HTTPS | Gratis durante toda la evaluación (hasta enero de 2027); una URL pública en el dominio de GoDaddy |

## Qué guarda MariaDB

- Análisis: contorno del lote, modo de ubicación (GPS o sitio de ejemplo), historial de cultivos, prioridades.
- Turnos de la conversación y el estado de cada apartado.
- Referencias a las fotos (los archivos viven fuera de la base de datos).
- Reportes generados, para el enlace compartible.
- Una caché de las respuestas de la NASA, para que el demo sea rápido y siga funcionando si un servicio está lento durante la evaluación.

## Fuentes de datos según acceso

| Acceso | Fuentes |
|--------|---------|
| Sin cuenta | NASA POWER; Planetary Computer (Sentinel-2, Landsat, NASADEM, MODIS, NEX-GDDP-CMIP6, ERA5); capas de mapa NASA GIBS; búsqueda NASA CMR |
| Token de Earthdata Login (del dueño, guardado en `env.txt`) | Humedad del suelo SMAP L4 (NSIDC), lluvia diaria IMERG Late (GES DISC), masas de agua GRACE/GRACE-FO (PO.DAAC) |

## Secretos

`env.txt` (claves de API) e `infra/.env` (contraseñas de la base de datos) están fuera de git y nunca se muestran en registros ni en el chat.

## Privacidad

Las ubicaciones y las fotos de cultivos son datos personales. Los análisis se borran tras un periodo de retención, salvo los sitios de ejemplo, y la app lo informa.

## Pendientes

- Confirmar la licencia de cada modelo NEX-GDDP-CMIP6 que se use.
- Extraer una vez la serie de GRACE por sitio de ejemplo (lector de netCDF en el backend o una herramienta puntual).
