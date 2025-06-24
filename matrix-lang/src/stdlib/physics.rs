// Physics Standard Library for Matrix Language
// Provides physics world creation, object management, and simulation functions

use crate::eval::{Interpreter, RuntimeError, Value};
use crate::stdlib::{PhysicsWorld, Vec3, PHYSICS_WORLDS};

pub fn register_physics_functions(interpreter: &mut Interpreter) {
    // Physics world creation
    interpreter.environment.define(
        "create_physics_world".to_string(),
        Value::BuiltinFunction {
            name: "create_physics_world".to_string(),
            arity: 0,
            func: |_args| {
                let world = PhysicsWorld::new();
                let world_id = world.id;

                let mut worlds = PHYSICS_WORLDS.lock().unwrap();
                worlds.insert(world_id, world);

                Ok(Value::Int(world_id as i64))
            },
        },
    );

    // Add rigid body to physics world
    interpreter.environment.define(
        "add_rigid_body".to_string(),
        Value::BuiltinFunction {
            name: "add_rigid_body".to_string(),
            arity: 4,
            func: |args| {
                if args.len() != 4 {
                    return Err(RuntimeError::TypeError {
                        message:
                            "add_rigid_body expects 4 arguments: (world_id, shape, mass, position)"
                                .to_string(),
                    });
                }

                let world_id = match &args[0] {
                    Value::Int(id) => *id as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "World ID must be integer".to_string(),
                        })
                    }
                };

                let shape = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Shape must be string".to_string(),
                        })
                    }
                };

                let mass = match &args[2] {
                    Value::Float(m) => *m,
                    Value::Int(m) => *m as f64,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Mass must be number".to_string(),
                        })
                    }
                };

                let position = match &args[3] {
                    Value::Array(arr) => {
                        if arr.len() != 3 {
                            return Err(RuntimeError::TypeError {
                                message: "Position must be [x, y, z] array".to_string(),
                            });
                        }
                        let x = match &arr[0] {
                            Value::Float(f) => *f,
                            Value::Int(i) => *i as f64,
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    message: "Position components must be numbers".to_string(),
                                })
                            }
                        };
                        let y = match &arr[1] {
                            Value::Float(f) => *f,
                            Value::Int(i) => *i as f64,
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    message: "Position components must be numbers".to_string(),
                                })
                            }
                        };
                        let z = match &arr[2] {
                            Value::Float(f) => *f,
                            Value::Int(i) => *i as f64,
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    message: "Position components must be numbers".to_string(),
                                })
                            }
                        };
                        Vec3 { x, y, z }
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Position must be [x, y, z] array".to_string(),
                        })
                    }
                };

                let mut worlds = PHYSICS_WORLDS.lock().unwrap();
                if let Some(world) = worlds.get_mut(&world_id) {
                    let object_id = world.add_object(shape, mass, position);
                    Ok(Value::Int(object_id as i64))
                } else {
                    Err(RuntimeError::Generic {
                        message: "Physics world not found".to_string(),
                    })
                }
            },
        },
    );

    // Step physics simulation
    interpreter.environment.define(
        "physics_step".to_string(),
        Value::BuiltinFunction {
            name: "physics_step".to_string(),
            arity: 1,
            func: |args| {
                if args.len() != 1 {
                    return Err(RuntimeError::TypeError {
                        message: "physics_step expects 1 argument: world_id".to_string(),
                    });
                }

                let world_id = match &args[0] {
                    Value::Int(id) => *id as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "World ID must be integer".to_string(),
                        })
                    }
                };

                let mut worlds = PHYSICS_WORLDS.lock().unwrap();
                if let Some(world) = worlds.get_mut(&world_id) {
                    world.step();
                    Ok(Value::Unit)
                } else {
                    Err(RuntimeError::Generic {
                        message: "Physics world not found".to_string(),
                    })
                }
            },
        },
    );

    // Get object position
    interpreter.environment.define(
        "get_object_position".to_string(),
        Value::BuiltinFunction {
            name: "get_object_position".to_string(),
            arity: 2,
            func: |args| {
                if args.len() != 2 {
                    return Err(RuntimeError::TypeError {
                        message: "get_object_position expects 2 arguments: (world_id, object_id)"
                            .to_string(),
                    });
                }

                let world_id = match &args[0] {
                    Value::Int(id) => *id as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "World ID must be integer".to_string(),
                        })
                    }
                };

                let object_id = match &args[1] {
                    Value::Int(id) => *id as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Object ID must be integer".to_string(),
                        })
                    }
                };

                let worlds = PHYSICS_WORLDS.lock().unwrap();
                if let Some(world) = worlds.get(&world_id) {
                    if let Some(object) = world.objects.get(object_id) {
                        Ok(Value::Array(vec![
                            Value::Float(object.position.x),
                            Value::Float(object.position.y),
                            Value::Float(object.position.z),
                        ]))
                    } else {
                        Err(RuntimeError::Generic {
                            message: "Physics object not found".to_string(),
                        })
                    }
                } else {
                    Err(RuntimeError::Generic {
                        message: "Physics world not found".to_string(),
                    })
                }
            },
        },
    );

    // Get object information (uses id, shape, mass fields)
    interpreter.environment.define(
        "get_object_info".to_string(),
        Value::BuiltinFunction {
            name: "get_object_info".to_string(),
            arity: 2,
            func: |args| {
                if args.len() != 2 {
                    return Err(RuntimeError::TypeError {
                        message: "get_object_info expects 2 arguments: (world_id, object_id)"
                            .to_string(),
                    });
                }

                let world_id = match &args[0] {
                    Value::Int(id) => *id as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "World ID must be integer".to_string(),
                        })
                    }
                };

                let object_id = match &args[1] {
                    Value::Int(id) => *id as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Object ID must be integer".to_string(),
                        })
                    }
                };

                let worlds = PHYSICS_WORLDS.lock().unwrap();
                if let Some(world) = worlds.get(&world_id) {
                    if let Some(object) = world.objects.get(object_id) {
                        use std::collections::HashMap;
                        let mut fields = HashMap::new();
                        fields.insert("id".to_string(), Value::Int(object.id as i64));
                        fields.insert("shape".to_string(), Value::String(object.shape.clone()));
                        fields.insert("mass".to_string(), Value::Float(object.mass));
                        fields.insert(
                            "position".to_string(),
                            Value::Array(vec![
                                Value::Float(object.position.x),
                                Value::Float(object.position.y),
                                Value::Float(object.position.z),
                            ]),
                        );
                        fields.insert(
                            "velocity".to_string(),
                            Value::Array(vec![
                                Value::Float(object.velocity.x),
                                Value::Float(object.velocity.y),
                                Value::Float(object.velocity.z),
                            ]),
                        );
                        fields.insert("is_static".to_string(), Value::Bool(object.is_static));

                        Ok(Value::Struct {
                            name: "PhysicsObject".to_string(),
                            fields,
                        })
                    } else {
                        Err(RuntimeError::Generic {
                            message: "Physics object not found".to_string(),
                        })
                    }
                } else {
                    Err(RuntimeError::Generic {
                        message: "Physics world not found".to_string(),
                    })
                }
            },
        },
    );

    // Set object mass (uses mass field)
    interpreter.environment.define(
        "set_object_mass".to_string(),
        Value::BuiltinFunction {
            name: "set_object_mass".to_string(),
            arity: 3,
            func: |args| {
                if args.len() != 3 {
                    return Err(RuntimeError::TypeError {
                        message: "set_object_mass expects 3 arguments: (world_id, object_id, mass)"
                            .to_string(),
                    });
                }

                let world_id = match &args[0] {
                    Value::Int(id) => *id as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "World ID must be integer".to_string(),
                        })
                    }
                };

                let object_id = match &args[1] {
                    Value::Int(id) => *id as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Object ID must be integer".to_string(),
                        })
                    }
                };

                let new_mass = match &args[2] {
                    Value::Float(m) => *m,
                    Value::Int(m) => *m as f64,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Mass must be number".to_string(),
                        })
                    }
                };

                let mut worlds = PHYSICS_WORLDS.lock().unwrap();
                if let Some(world) = worlds.get_mut(&world_id) {
                    if let Some(object) = world.objects.get_mut(object_id) {
                        object.mass = new_mass;
                        object.is_static = new_mass == 0.0;
                        Ok(Value::Unit)
                    } else {
                        Err(RuntimeError::Generic {
                            message: "Physics object not found".to_string(),
                        })
                    }
                } else {
                    Err(RuntimeError::Generic {
                        message: "Physics world not found".to_string(),
                    })
                }
            },
        },
    );

    // Get object mass (uses mass field)
    interpreter.environment.define(
        "get_object_mass".to_string(),
        Value::BuiltinFunction {
            name: "get_object_mass".to_string(),
            arity: 2,
            func: |args| {
                if args.len() != 2 {
                    return Err(RuntimeError::TypeError {
                        message: "get_object_mass expects 2 arguments: (world_id, object_id)"
                            .to_string(),
                    });
                }

                let world_id = match &args[0] {
                    Value::Int(id) => *id as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "World ID must be integer".to_string(),
                        })
                    }
                };

                let object_id = match &args[1] {
                    Value::Int(id) => *id as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Object ID must be integer".to_string(),
                        })
                    }
                };

                let worlds = PHYSICS_WORLDS.lock().unwrap();
                if let Some(world) = worlds.get(&world_id) {
                    if let Some(object) = world.objects.get(object_id) {
                        Ok(Value::Float(object.mass))
                    } else {
                        Err(RuntimeError::Generic {
                            message: "Physics object not found".to_string(),
                        })
                    }
                } else {
                    Err(RuntimeError::Generic {
                        message: "Physics world not found".to_string(),
                    })
                }
            },
        },
    );

    // Get object shape (uses shape field)
    interpreter.environment.define(
        "get_object_shape".to_string(),
        Value::BuiltinFunction {
            name: "get_object_shape".to_string(),
            arity: 2,
            func: |args| {
                if args.len() != 2 {
                    return Err(RuntimeError::TypeError {
                        message: "get_object_shape expects 2 arguments: (world_id, object_id)"
                            .to_string(),
                    });
                }

                let world_id = match &args[0] {
                    Value::Int(id) => *id as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "World ID must be integer".to_string(),
                        })
                    }
                };

                let object_id = match &args[1] {
                    Value::Int(id) => *id as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Object ID must be integer".to_string(),
                        })
                    }
                };

                let worlds = PHYSICS_WORLDS.lock().unwrap();
                if let Some(world) = worlds.get(&world_id) {
                    if let Some(object) = world.objects.get(object_id) {
                        Ok(Value::String(object.shape.clone()))
                    } else {
                        Err(RuntimeError::Generic {
                            message: "Physics object not found".to_string(),
                        })
                    }
                } else {
                    Err(RuntimeError::Generic {
                        message: "Physics world not found".to_string(),
                    })
                }
            },
        },
    );

    // List all objects in world (uses id field)
    interpreter.environment.define(
        "list_objects".to_string(),
        Value::BuiltinFunction {
            name: "list_objects".to_string(),
            arity: 1,
            func: |args| {
                if args.len() != 1 {
                    return Err(RuntimeError::TypeError {
                        message: "list_objects expects 1 argument: world_id".to_string(),
                    });
                }

                let world_id = match &args[0] {
                    Value::Int(id) => *id as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "World ID must be integer".to_string(),
                        })
                    }
                };

                let worlds = PHYSICS_WORLDS.lock().unwrap();
                if let Some(world) = worlds.get(&world_id) {
                    let object_ids: Vec<Value> = world
                        .objects
                        .iter()
                        .map(|obj| Value::Int(obj.id as i64))
                        .collect();
                    Ok(Value::Array(object_ids))
                } else {
                    Err(RuntimeError::Generic {
                        message: "Physics world not found".to_string(),
                    })
                }
            },
        },
    );
}
