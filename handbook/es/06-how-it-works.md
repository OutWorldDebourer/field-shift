# 6. Cómo funciona y cómo ejecutarlo

Construido durante el hackathon (14 y 15 de noviembre de 2026). Código: `backend/` (Rust), `frontend/` (TypeScript), `infra/` (MariaDB).

## El flujo

1. **Campo.** El agricultor usa su GPS, un sitio de ejemplo (valle de Huaral, Perú), la búsqueda de lugares, un punto o un contorno dibujado. El área del contorno se calcula sobre la esfera.
2. **Conversación.** El consultor (Claude, modelo `claude-opus-5-5`, vía la API de Anthropic) pregunta qué se cultiva ahora y qué hubo antes, la fuente de agua y el último abono, una o dos preguntas a la vez, y guía las fotos (hasta 10 por mensaje). "No sé" se registra como un vacío declarado.
3. **Etapa de análisis (paquete de evidencia).** El backend lee en paralelo NASA POWER (clima desde 1981 y su normal 1985–2024, evapotranspiración de referencia FAO-56, balance hídrico, grados-día y generaciones de plagas), el vigor del cultivo de Sentinel-2 y Landsat dentro del contorno (Microsoft Planetary Computer), la pendiente NASADEM, la humedad del suelo SMAP, la lluvia IMERG, el agua almacenada GRACE y la proyección NEX-GDDP-CMIP6. Cada valor lleva fuente, fecha, resolución, licencia y una etiqueta de escala: **nivel de campo** (10–30 m) o **contexto regional** (POWER ~50 km, SMAP 9 km, IMERG 10 km, GRACE ~300 km).
4. **Conocimiento de cultivos.** Cualquier cultivo que nombre el agricultor se normaliza a una ficha (nombres, nombre científico, familia, tipo: anual, semiperenne o perenne). Una base semilla verificada responde al instante los cultivos conocidos; los demás reciben una ficha investigada con búsqueda web, cada valor con su cita, marcada "investigado por IA, verificar".
5. **Consejos.** El tipo de cultivo decide: los anuales reciben tres estrategias de rotación (incluida la actual) puntuadas en suelo, agua, corte de plagas, ingreso y riesgo; los semiperennes, renovación y cobertura intermedia; los perennes nunca reciben "rotar el cultivo principal", sino cobertura entre hileras, asociaciones, manejo de rastrojos y control biológico. Suelo, sanidad (insectos y hongos benéficos como *Chrysoperla*, *Trichoderma*, *Beauveria*) y nutrición siguen la entrevista con la agrónoma.
6. **El consultor** solo cita números del paquete de evidencia, primero en palabras simples. Todo es un **prediagnóstico**; se deriva al agrónomo.
7. **Exportar.** Una ficha técnica imprimible (guardar como PDF) con cada lectura, fuente, fecha y licencia; un CSV con todas las series; un enlace de solo lectura para compartir.

Si el modelo en vivo no está disponible o un visitante alcanza el límite (30 turnos en vivo por hora, 400 por día en total), el consultor pasa a un **modo demo guionizado** claramente señalado que nunca finge leer fotos.

## Ejecutarlo en local

```bash
docker compose -f infra/compose.yaml --env-file infra/.env up -d
```

```bash
cd backend && cargo run --bin fieldshift
```

```bash
cd frontend && npm install && npx vite
```

Abre http://localhost:5173 (inicio) y http://localhost:5173/app.html (app). El backend lee las claves de `env.txt` en la raíz del proyecto (líneas `api.nasa.gov`, `earthdata`, `anthropic`) y la base de datos de `infra/.env`; ninguno de los dos archivos está en git.

## Servicios y endpoints (todos bajo `/api`)

| Área | Endpoints |
|------|-----------|
| Salud | `GET /health` |
| Análisis | `POST /analyses`, `GET /analyses/examples`, `GET /analyses/{id}`, `POST /analyses/{id}/evidence` |
| Consultor | `POST /consultant/{id}/message` (multipart: text, lang, photos) |
| Clima | `GET /power/summary`, `/power/series`, `/power/monthly` |
| Imágenes | `GET /imagery/ndvi`, `/imagery/scenes`, `/imagery/terrain`, `/imagery/snapshot.png` |
| Earthdata | `GET /earthdata/smap`, `/earthdata/imerg`, `/earthdata/grace` |
| Proyección | `GET /climate/projection` |
| Cultivos | `GET /crops/suggest`, `/crops/resolve`, `POST /crops/advice` |
| Exportar | `GET /export/{id}/sheet`, `/export/{id}/series.csv`, `POST /export/{id}/share` |

## Iteración 2 (14-11-2026)

- **Temas guiados.** El consultor recorre cinco temas en orden (tu campo, suelo, manejo de plagas, nutrición, rotación); cada sección pasa a "lista" solo cuando su tema se completa, el chat muestra el siguiente tema y lo que falta, y una confirmación breve marca cada tema completado. Al terminar, el consultor lo dice y remite a la ficha técnica.
- **Saludo.** El primer mensaje dice dónde está el campo (latitud, longitud, área) y aclara que los datos describen la parcela, no a la persona.
- **Pronóstico, perspectiva y proyección (solo NASA).** Próximos 1–10 días con NASA GEOS-FP (pronóstico); los próximos 1–6 meses como estimación con las normales de NASA POWER en dos escenarios (vuelve a lo normal / sigue así de cálido); 2031–2050 con NEX-GDDP-CMIP6 (proyección). El panel permite cambiar cada horizonte.
- **Dictado por voz.** Un micrófono llena el cuadro de mensaje sin enviarlo; motores: Gemini (por defecto, `POST /api/consultant/transcribe`) o el reconocimiento de voz del navegador.
- **Fotos en resolución completa.** Se guardan hasta 12 MB; el modelo recibe además los cuatro cuartos de cada foto grande en resolución nativa para que los insectos pequeños se vean.
- **Ficha técnica.** Enlace para volver al análisis, fotos del agricultor, imagen satelital, pronóstico y perspectiva, anexo de glosario y un paquete ZIP con los originales.

## Iteración 3 (14-11-2026)

- **Documentos.** El chat acepta PDF (hasta 15 MB) además de fotos. El agricultor puede adjuntar la ficha técnica de una visita anterior: el consultor la lee, dice qué tomó de ella y su fecha, registra las respuestas y completa los temas que cubre; las lecturas de la NASA siempre son nuevas.
- **Cargar información.** Un formulario con lo que necesita cada tema (cultivo actual y anteriores, riego, suelo, materia orgánica, rastrojos, plagas vistas, productos recientes, último abono, análisis de suelo, próximo cultivo, destino de la cosecha; se permite "no sé"). En el sitio de ejemplo se abre con respuestas de ejemplo señaladas, para que los jurados vean todas las secciones llenas. `POST /api/analyses/{id}/intake`.
- **Pestaña Orgánico.** Manejo biológico y ecológico de plagas: enemigos naturales y microorganismos benéficos (insectos, ácaros, hongos, bacterias) con su fuente publicada, primero los que coinciden con las plagas reportadas (buscados en toda la base de cultivos); prácticas que los mantienen; cuándo actuar según grados-día NASA y el pronóstico GEOS-FP; qué los daña. Sin dosis: eso lo decide el agrónomo.
- **Lo que nos contó el agricultor** aparece en la sección Tu campo y en la ficha técnica.


## Iteración 4 (2026-11-14)

- **Plagas honestas.** Las generaciones de plagas, las alertas, el mosaico de plagas, los consejos de sanidad, las señales de rotación y el momento de acción orgánico usan los cultivos del propio agricultor (actual, anteriores, próximo y propuesto), cada plaga con su umbral publicado de grados-día; las que no lo tienen reciben una guía cualitativa. Las tres plagas de referencia general solo aparecen mientras no se conoce el cultivo, con la etiqueta "referencia general, no tu cultivo".
- **Inicio limpio y una sola ficha.** "Iniciar análisis" abre un mapa vacío ("Usar mi último campo" recupera el anterior solo si lo pides). Un análisis nuevo abre primero la ficha de una pantalla; al enviarla se completan todos los temas, secciones y la ficha técnica sin ninguna llamada al modelo. "Omitir, prefiero conversar" abre la conversación, y el consultor pide lo que falta en una sola lista.
- **Consultor ligero.** Los resultados de las herramientas se compactan en el idioma de la conversación, la cola de la conversación se guarda en caché y los tokens de cada llamada quedan en `api_usage`. Una sonda gratuita (`token_probe`, `count_tokens` de Anthropic) mide un turno guionizado de Huaral: 39 850 tokens de entrada antes, unos 26 500 después (con más herramientas que antes).
- **Chat compacto y burbuja flotante.** Encabezado de una línea con los cultivos, barra de progreso de seis puntos, respuestas rápidas que solo llenan el cuadro, compositor de una fila. Fuera de la pestaña Chat, el chat se vuelve una burbuja que abre un mini chat sobre la vista actual (el mismo chat, así el borrador se conserva): "Preguntando sobre: Rotación · estrategia 2".
- **El consultor conoce cada vista.** `get_view` devuelve lo que muestra cada pestaña con los mismos números; cada turno lleva la pestaña y la tarjeta sobre la que pregunta el agricultor ("Preguntar sobre esto").
- **Orgánico por categorías y "¿Y si siembro…?".** Orgánico empieza con tres líneas para esta semana y luego tarjetas por categoría, con el detalle técnico a un clic. En Rotación, cualquier cultivo que escriba el agricultor se vuelve una cuarta columna puntuada con el mismo motor; los perennes se evalúan como plantación. El consultor hace lo mismo con `evaluate_crop`.
- **Prevención.** Para cada cultivo de la conversación, cada plaga recibe un semáforo (alto / medio / bajo / sin datos) con una razón simple, según los grados-día NASA POWER frente a lo normal, el pronóstico NASA GEOS-FP de 10 días y el clima que favorece a la plaga. Los cinco cultivos de Huaral (arveja, lechuga, camote, maíz, papa) tienen condiciones de plagas, micronutrientes y bioestimulantes investigados y citados, marcados "investigado por IA, verificar" hasta su revisión; los demás cultivos muestran el vacío. Sin nombres de productos ni dosis.
