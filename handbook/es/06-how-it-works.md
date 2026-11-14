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

