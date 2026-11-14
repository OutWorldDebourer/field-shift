# 5. Ofertas Globales

Leído el 2026-11-14 en la página de ofertas con sesión iniciada: <https://www.spaceappschallenge.org/2026/global-offers/>

## Cómo reclamar

Iniciar sesión en spaceappschallenge.org, clic en el avatar, elegir **Offers**, abrir la oferta y presionar **Claim Offer**. Cada botón abre el sitio del proveedor; nada se activa dentro de Space Apps. Todas las ofertas son opcionales. Las dudas van al contacto de soporte de cada oferta o a la categoría Global Offers de Space Apps Connect: <https://github.com/nasa/spaceapps/discussions/categories/global-offers>

## Las ofertas de un vistazo

| Oferta | Qué recibes | Pide tarjeta | Enlace para reclamar |
|--------|-------------|--------------|----------------------|
| GoDaddy Registry: dominio y hosting | Un dominio no premium gratis, un año de hosting web y certificado SSL por ese año | No se indica | <https://www.spaceapps.earth/> |
| Google Cloud | La prueba gratuita estándar: 300 USD de crédito por 90 días, sin cobro automático después | Sí, para verificar identidad | <https://cloud.google.com/free/docs/free-cloud-features#free-trial> |
| Microsoft Azure | La cuenta gratuita estándar para clientes nuevos: 200 USD de crédito por 30 días más los servicios siempre gratuitos y los gratuitos por 12 meses | Sí, no prepago, más un teléfono | <https://azure.microsoft.com/en-us/free/> |
| Microsoft Planetary Computer | Catálogo de datos y API STAC públicos, sin cuenta; algunas solicitudes piden un token gratuito | No | <https://planetarycomputer.microsoft.com/> |
| GitHub Copilot | El plan Free: hasta 2 000 autocompletados al mes, chat y agente limitados | No | <https://github.com/features/copilot> |
| Microsoft Copilot | Asistente gratuito en copilot.com | No | <https://www.microsoft.com/en-us/microsoft-copilot> |
| Microsoft Teams | Teams Free: 100 participantes, reuniones de 60 minutos, 5 GB por usuario | No | <https://www.microsoft.com/en-us/microsoft-teams/teams-free> |
| Meteomatics Weather API | 1 000 consultas diarias en todas las funciones de la API, **solo mientras dura el hackatón** | No | <https://www.meteomatics.com/en/meteomatics-and-nasa-space-apps-challenge/> |
| Miro Education | Gratis dos años para estudiantes | No | <https://miro.com/education-whiteboard/> |
| Miro Business | Prueba de 14 días; cancelar antes para evitar cobros | Sí | <https://miro.com/pricing/> |
| Plantillas de Miro | Tableros Hackathon y Project Coach | No | <https://miro.com/templates/profile/nasa-space-apps-challenge/> |

## Dominio de GoDaddy, paso a paso

Es la única oferta con un flujo propio. La página describe cuatro pasos:

1. Abrir <https://www.spaceapps.earth/> y buscar el nombre de dominio. Solo los dominios no premium son gratis.
2. Llenar los datos de contacto con un correo válido.
3. Revisar el correo e ingresar el "launch code" que llega.
4. Terminar el registro gratuito con **Porkbun**, el socio registrador de GoDaddy.

Soporte: support@porkbun.com. La página no dice qué extensiones incluye; el sitio para reclamar está en `.earth`, así que esa es la extensión probable (sin confirmar).

Completar el flujo crea una cuenta en Porkbun e ingresa datos personales de contacto, así que lo hace el dueño del equipo en persona.

## Qué significa para el proyecto

- **Google Cloud y Azure son las pruebas gratuitas públicas de siempre.** Son solo para clientes nuevos: una cuenta que ya usó su prueba no recibe crédito nuevo. Ambas piden tarjeta.
- **Planetary Computer no requiere reclamar nada.** Su API STAC es la vía listada más rápida a Landsat, Sentinel-2 y MODIS sin crear cuentas.
- **Meteomatics se corta cuando termina el hackatón**, y la evaluación dura hasta enero de 2027. Un demo público que dependa de él se rompería ante los jueces. Además son datos propietarios, no de la NASA. Usarlo como mucho para una comparación puntual, guardando el resultado.
- **Un dominio gratis le da al proyecto una URL pública limpia** para el campo "Link to Final Project". A un dominio nuevo conviene pedirle la clasificación de FortiGuard: <https://www.fortiguard.com/faq/wfratingsubmit>
- El enlace del proyecto final debe abrir sin login, y todo servicio que no sea de la NASA debe figurar en las referencias con su licencia.

## Aún abierto

- [ ] Extensiones de dominio y precio de renovación tras el año gratis.
- [ ] Si el hosting es obligatorio o el dominio puede apuntar a otro lado por DNS.
- [ ] Si las cuentas de Google y Microsoft del dueño del equipo aún califican para los créditos de cliente nuevo.

## Fuentes

- Página de Ofertas Globales (requiere login): <https://www.spaceappschallenge.org/2026/global-offers/>
- Mensaje de inicio del GO Team, guardado en `Docs/Correo.txt`
- Discusión 2035 de Space Apps Connect: <https://github.com/nasa/spaceapps/discussions/2035>
- Guía de envío del proyecto: <https://www.spaceappschallenge.org/resources/project-submission-guide/>
