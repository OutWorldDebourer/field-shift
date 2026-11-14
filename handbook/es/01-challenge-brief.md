# 1. Resumen del reto

**Evento:** NASA Space Apps Challenge 2026, 14 y 15 de noviembre de 2026
**Reto:** Field Shift: Adapting Farms with NASA Data (Desplazamiento de campo: adaptando granjas con datos de la NASA)
**Dificultad:** Intermedia / Avanzada. **Temas:** Ciencias de la Tierra, Software, Artes y Multimedia
**Equipo:** InnoHuando
**Página oficial:** <https://www.spaceappschallenge.org/2026/challenges/adapting-farms-with-nasa-data/>

## El pedido, en una frase

Construir una herramienta de apoyo a la decisión que combine observaciones terrestres de la NASA con información local de suelo, características de los cultivos y prioridades del agricultor, para que este pueda explorar estrategias de rotación que fortalezcan la salud del suelo y adapten la chacra a los cambios de clima y agua.

## Qué pide realmente el enunciado

Todo el enunciado está redactado como "puedes", así que nada de lo siguiente es obligatorio. Leído en conjunto, sin embargo, describe cómo se ve una buena respuesta.

| # | Elemento | Dónde aparece |
|---|----------|---------------|
| 1 | Usar observaciones terrestres de la NASA como insumo central (SMAP, ECOSTRESS, GPM, Landsat, MODIS, NASA POWER, GRACE/GRACE-FO, SRTM) | Objetivos |
| 2 | Analizar temperatura, estrés térmico, lluvia, humedad del suelo, evaporación, riesgo de heladas, lluvias extremas, estrés vegetal y tendencias hídricas de largo plazo | Objetivos |
| 3 | Combinar con datos locales de suelo, rasgos de cultivos y prioridades del agricultor (uso de agua, valor forrajero, fijación de nitrógeno, reducción de erosión, materia orgánica, mercado) | Objetivos |
| 4 | Mostrar cuándo los campos suelen estar desnudos y cuándo los cultivos actuales se ven más vulnerables | Objetivos |
| 5 | **Comparar al menos tres estrategias de rotación, una de ellas la rotación actual** | Objetivos |
| 6 | Para cada estrategia explicar: por qué se sugirió, la evidencia, los beneficios, los vacíos de información y los riesgos que requieren revisión local | Objetivos |
| 7 | Ser una herramienta de apoyo: nunca reemplaza el juicio del agricultor ni la experiencia local | Objetivos |

Los puntos 5, 6 y 7 son el corazón del reto. Una herramienta que solo muestra mapas y gráficos no lo responde.

## Consideraciones opcionales, agrupadas como en el enunciado

- **Entender las condiciones y los datos.** Explicar en lenguaje simple términos como evapotranspiración, anomalía de humedad del suelo, NDVI, riesgo de heladas y balance hídrico. Explicar en qué se diferencian las observaciones satelitales de las mediciones en tierra.
- **Modelado y fusión de datos.** Comparar enfoques estadísticos o de aprendizaje automático, decir cómo se maneja la incertidumbre y armonizar datos de distinta resolución en una sola grilla de análisis.
- **Exploración de estrategias de rotación.** Comparar escenarios según ciclo de nutrientes, erosión, materia orgánica, ruptura del ciclo de plagas, uso de agua, forraje y mercado. Relacionar las estrategias con las prioridades del agricultor. Explicar el papel de leguminosas, cultivos de cobertura, de raíz profunda y tolerantes a sequía.
- **Mapas y visualización.** Mapas interactivos de tendencias de largo plazo, zonas de vulnerabilidad a nivel de campo, líneas de tiempo, capas superpuestas, deslizadores de escenarios de lluvia o temperatura.
- **Diseño centrado en el agricultor.** Entendible para personas con poca formación técnica, permite ingresar prioridades y restricciones (agua, maquinaria, mercado) y recuerda dónde importa el criterio humano.

## Nota de vocabulario

El sitio de Space Apps escribe "SI" (Super Intelligence) donde casi todos escriben "IA". Su FAQ indica que sigue una Orden Ejecutiva de EE. UU. del 29 de septiembre de 2026. Usamos el término del sitio en el formulario de envío y "IA/ML" en nuestras notas técnicas.

## Qué ofrece la pestaña oficial de Recursos

La pestaña lista 24 recursos. Se dividen en tres grupos, y esa división pesa en dónde hacemos el piloto.

**Cobertura global (sirven en cualquier lugar, incluido Perú)**

| Recurso | Qué aporta | Enlace |
|---------|------------|--------|
| SMAP | Humedad superficial del suelo y congelamiento cada 2-3 días | <https://www.earthdata.nasa.gov/data/platforms/space-based-platforms/smap> |
| GPM / IMERG | Tasa y acumulado de precipitación | <https://gpm.nasa.gov/data> |
| Landsat | Reflectancia a 30 m, NDVI, suelo desnudo | <https://earthexplorer.usgs.gov/> |
| MODIS / VIIRS | Índices de vegetación a 250-1000 m | <https://lpdaac.usgs.gov/products/> |
| NASA POWER | Agroclimatología: temperatura, humedad, radiación, indicadores derivados | <https://power.larc.nasa.gov/> |
| SRTM | Elevación, pendiente, escorrentía, riesgo de erosión | <https://www.earthdata.nasa.gov/data/instruments/srtm> |
| ECOSTRESS | Temperatura de superficie y evapotranspiración a escala fina | <https://ecostress.jpl.nasa.gov/data> |
| GLDAS | Temperatura, lluvia, humedad del suelo y ET asimiladas | <https://www.earthdata.nasa.gov/> |
| GEDI | Altura de dosel y biomasa (mejor en vegetación alta) | <https://gedi.umd.edu/data> |
| FAO GAEZ | Aptitud de cultivos, limitantes de suelo, rendimiento potencial | <https://gaez.fao.org/> |
| Copernicus Sentinel-2 (ESA) | Imágenes multiespectrales de 10-20 m | <https://dataspace.copernicus.eu/> |
| ERA5 (ECMWF) | Variables climáticas horarias de reanálisis | <https://cds.climate.copernicus.eu/datasets/reanalysis-era5-single-levels> |

**Solo Estados Unidos**

USDA Ag Data Commons, Web Soil Survey, Cropland Data Layer (CropScape), PRISM, SSURGO/STATSGO2, NAIP, NLCD, NLDAS, NWIS, USDA AMS Market News, NASS QuickStats, OpenET (estados del oeste), LANDFIRE.

**Solo Norteamérica**

CoCoRaHS, lluvia reportada por voluntarios.

### Por qué importa

Más de la mitad de los recursos listados cubre solo Estados Unidos. Un piloto en Perú puede usar todos los datos de misiones de la NASA, pero debe reemplazar las fuentes estadounidenses de suelo, mapa de cultivos, precios y estaciones por equivalentes peruanos o globales. Ver [6. Decisiones abiertas](06-open-decisions.md).

## Fuentes

- Enunciado del reto y pestaña de Recursos, leídos el 2026-11-14: <https://www.spaceappschallenge.org/2026/challenges/adapting-farms-with-nasa-data/?tab=resources>
- Preguntas frecuentes: <https://www.spaceappschallenge.org/resources/participant-faqs/>
