use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use warp::{http::StatusCode, Filter};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ResultadoCalculo {
    id: String,
    resultado_soma: f64,
    resultado_subtracao: f64,
}

#[derive(Deserialize, Debug)]
struct CalculoRequisicao {
    id: String,
    valor1: f64,
    valor2: f64,
    valor3: f64,
}

type CalculosMap = Arc<Mutex<HashMap<String, ResultadoCalculo>>>;

async fn processar_calculo(req: CalculoRequisicao, calculos: CalculosMap) {
    let resultado = calcular(&req, &calculos);

    {
        let calculos = calculos.lock().unwrap(); // Trava o Mutex para acesso seguro
        println!("ID\tResultadoSoma\tResultadoSubtracao");
        for (id, calculo) in calculos.iter() {
            println!("{}\t{}\t{}", id, calculo.resultado_soma, calculo.resultado_subtracao);
        }
    }

}

fn calcular(req: &CalculoRequisicao, calculos: &CalculosMap) -> ResultadoCalculo {
    let mut calculos = calculos.lock().unwrap();
    let ultimo_resultado = calculos.get(&req.id).cloned().unwrap_or(ResultadoCalculo {
        id: req.id.clone(),
        resultado_soma: 0.0,
        resultado_subtracao: 0.0,
    });

    let novo_resultado = ResultadoCalculo {
        id: req.id.clone(),
        resultado_soma: ultimo_resultado.resultado_soma + req.valor1 + req.valor2,
        resultado_subtracao: ultimo_resultado.resultado_subtracao - (req.valor2 + req.valor3),
    };

    calculos.insert(req.id.clone(), novo_resultado.clone());
    novo_resultado
}


async fn adicionar_calculo(calculo: CalculoRequisicao, sender: mpsc::Sender<CalculoRequisicao>) -> Result<impl warp::Reply, warp::Rejection> {
    if sender.send(calculo).await.is_ok() {
        Ok(warp::reply::with_status("Calculo aceito", StatusCode::ACCEPTED))
    } else {
        Err(warp::reject::reject())
    }
}

#[tokio::main]
async fn main() {
    let calculos = Arc::new(Mutex::new(HashMap::new()));
    let (sender, mut receiver) = mpsc::channel::<CalculoRequisicao>(100);

    let calculos_for_worker = calculos.clone();
    tokio::spawn(async move {
        while let Some(req) = receiver.recv().await {
            let calculos_clone = calculos_for_worker.clone();
            tokio::spawn(async move {
                processar_calculo(req, calculos_clone).await;
            });
        }
    });

    let calculos_route = warp::post()
        .and(warp::path("calculo"))
        .and(warp::body::json())
        .and(warp::any().map(move || sender.clone()))
        .and_then(adicionar_calculo);

    warp::serve(calculos_route)
        .run(([127, 0, 0, 1], 8080))
        .await;
}
