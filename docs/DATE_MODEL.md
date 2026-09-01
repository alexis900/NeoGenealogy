# Date model

`DateValue` conserva siempre `raw` y expone `precision`: `Exact`, `Year`, `About`, `Before`, `After`, `Between`, `FromTo` o `Unknown`. Los intervalos mantienen `start_year` y `end_year`.

No se transforma una fecha aproximada en el primer día del año. El modelo reserva espacio para calendario y conversión histórica futura.
