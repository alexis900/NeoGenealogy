# Analysis rules

Los hallazgos son señales de investigación, no afirmaciones de hechos. Cada resultado conserva evidencia y `confidence` entre 0 y 1.

| Tipo | Criterio | Severidad |
|---|---|---|
| `missing-data` | Falta un campo biográfico | `Info` |
| `chronology` | Nacimiento posterior a defunción | `High` |
| `AGE_ANOMALY` | Progenitor menor de 14 o mayor de 55 años al nacer el hijo | `High`/`Warning` |
| `RELATIONSHIP_ANOMALY` | Hijo nacido antes del matrimonio registrado | `Warning` |
| `POSSIBLE_DUPLICATE` | Coinciden al menos dos señales de nombre, año o lugar | `Warning` |
| `research-gap` | Persona sin familia de origen conocida | `High` |
| `missing-source` | Persona sin citas de fuente | `Medium` |

Los umbrales de edad se pueden cambiar mediante `AnalysisConfig`. Las edades inusuales no se convierten automáticamente en errores.
