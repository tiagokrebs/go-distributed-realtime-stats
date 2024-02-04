package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"sync"
)

type ResultadosCalculos struct {
	ID                 string
	ResultadoSoma      float64
	ResultadoSubtracao float64
}

type CalculoRequisicao struct {
	ID     string
	Valor1 float64
	Valor2 float64
	Valor3 float64
}

var (
	calculosPorID sync.Map               // Usando sync.Map para segurança concorrente
	requisicoes   chan CalculoRequisicao = make(chan CalculoRequisicao, 100)
	numWorkers    int                    = 5
)

func worker() {
	for req := range requisicoes {
		resultados := calcular(req.ID, req.Valor1, req.Valor2, req.Valor3)
		calculosPorID.Store(req.ID, resultados) // Armazena usando sync.Map

		// Itera sobre sync.Map para imprimir seu estado atual
		fmt.Println("ID\tResultadoSoma\tResultadoSubtracao")
		calculosPorID.Range(func(key, value interface{}) bool {
			id := key.(string)
			resultados := value.(ResultadosCalculos)
			fmt.Printf("%s\t%f\t%f\n", id, resultados.ResultadoSoma, resultados.ResultadoSubtracao)
			return true // Continua a iteração
		})
	}
}

func calcular(id string, valor1, valor2, valor3 float64) ResultadosCalculos {
	valor, ok := calculosPorID.Load(id) // Carrega usando sync.Map
	var ultimoResultado ResultadosCalculos
	if ok {
		ultimoResultado = valor.(ResultadosCalculos)
	} else {
		ultimoResultado = ResultadosCalculos{ID: id}
	}

	soma := ultimoResultado.ResultadoSoma + (valor1 + valor2)
	subtracao := ultimoResultado.ResultadoSubtracao - (valor3 - valor2)

	return ResultadosCalculos{ID: id, ResultadoSoma: soma, ResultadoSubtracao: subtracao}
}

func adicionarCalculo(w http.ResponseWriter, r *http.Request) {
	if r.Method != "POST" {
		http.Error(w, "Método não suportado", http.StatusMethodNotAllowed)
		return
	}

	var entrada CalculoRequisicao
	err := json.NewDecoder(r.Body).Decode(&entrada)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	requisicoes <- entrada
	w.WriteHeader(http.StatusAccepted)
}

func main() {
	for i := 0; i < numWorkers; i++ {
		go worker()
	}

	http.HandleFunc("/calculo", adicionarCalculo)

	fmt.Println("Servidor rodando na porta 8080...")
	log.Fatal(http.ListenAndServe(":8080", nil))
}
