use neon::prelude::*;
use neon::types::JsPromise;
use hadamard_core::HadamardEngine;
use std::collections::HashMap;
use std::sync::Arc;

pub struct NeonHadamardWrapper {
    pub engine: Arc<HadamardEngine>,
}

// Finalize implementation to ensure Node.js Garbage Collector sweeps memory safely
impl Finalize for NeonHadamardWrapper {}

/// Asynchronously handles the Node.js payload migration down to Tokio background runtime worker threads
fn js_new_engine_instance(mut cx: FunctionContext) -> JsResult<JsBox<NeonHadamardWrapper>> {
    let shard_capacity = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize;
    let max_qubits = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;

    let wrapper = NeonHadamardWrapper {
        engine: Arc::new(HadamardEngine::new(shard_capacity, max_qubits)),
    };

    Ok(cx.boxed(wrapper))
}

/// Dispatches raw JSON dataset string arrays into structural QRAM sharding layers from Node.js
fn js_load_dataset(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let wrapper = cx.argument::<JsBox<NeonHadamardWrapper>>(0)?;
    let collection_name = cx.argument::<JsString>(1)?.value(&mut cx);
    let js_array = cx.argument::<JsArray>(2)?;

    let mut dataset: Vec<HashMap<String, String>> = Vec::new();
    let length = js_array.len(&mut cx);

    // Marshal structural JavaScript Objects into bare-metal Rust HashMaps
    for i in 0..length {
        let obj: Handle<JsObject> = js_array.get(&mut cx, i)?.downcast_or_throw(&mut cx)?;
        let mut map = HashMap::new();
        
        let keys: Handle<JsArray> = obj.get_own_property_names(&mut cx)?;
        let keys_length = keys.len(&mut cx);
        
        for j in 0..keys_length {
            let key_js: Handle<JsString> = keys.get(&mut cx, j)?.downcast_or_throw(&mut cx)?;
            let key_str = key_js.value(&mut cx);
            let val_js: Handle<JsString> = obj.get(&mut cx, key_js)?.downcast_or_throw(&mut cx)?;
            map.insert(key_str, val_js.value(&mut cx));
        }
        dataset.push(map);
    }

    let (deferred, promise) = cx.promise();
    let channel = cx.channel();
    let engine_ref = wrapper.engine.clone();

    // Spawning background worker thread execution paths bypassing the main Node.js event loop
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let result = runtime.block_on(async {
            engine_ref.load_dataset(&collection_name, dataset).await
        });

        channel.send(move |mut cx| {
            match result {
                Ok(total_shards) => {
                    let js_shards = cx.number(total_shards as f64);
                    deferred.resolve(&mut cx, js_shards);
                }
                Err(e) => {
                    let err_msg = cx.string(format!("TypeScript Binding Ingestion Error: {}", e));
                    deferred.reject(&mut cx, err_msg);
                }
            }
            Ok(())
        });
    });

    Ok(promise)
}

/// Executes a compiled quantum query state directly over non-blocking channel streams
fn js_execute_query(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let wrapper = cx.argument::<JsBox<NeonHadamardWrapper>>(0)?;
    let sql_query = cx.argument::<JsString>(1)?.value(&mut cx);

    let (deferred, promise) = cx.promise();
    let channel = cx.channel();
    let engine_ref = wrapper.engine.clone();

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let result = runtime.block_on(async {
            engine_ref.query(&sql_query).await
        });

        channel.send(move |mut cx| {
            match result {
                Ok(records) => {
                    let js_result_array = cx.empty_array();
                    for (i, record) in records.iter().enumerate() {
                        let js_obj = cx.empty_object();
                        for (k, v) in record {
                            let k_js = cx.string(k);
                            let v_js = cx.string(v);
                            js_obj.set(&mut cx, k_js, v_js)?;
                        }
                        js_result_array.set(&mut cx, i as u32, js_obj)?;
                    }
                    deferred.resolve(&mut cx, js_result_array);
                }
                Err(e) => {
                    let err_msg = cx.string(format!("Quantum Query Pipeline crashed: {}", e));
                    deferred.reject(&mut cx, err_msg);
                }
            }
            Ok(())
        });
    });

    Ok(promise)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("createEngine", js_new_engine_instance)?;
    cx.export_function("loadDataset", js_load_dataset)?;
    cx.export_function("executeQuery", js_execute_query)?;
    Ok(())
}
