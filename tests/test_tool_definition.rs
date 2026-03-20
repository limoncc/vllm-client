//! Phase 4.1: Tool Definition Tests
//!
//! 测试工具定义的 JSON 格式

use serde_json::json;

// ============================================================================
// Test: test_tool_json_format
// 输入: json!({"type": "function", "function": {...}})
// 预期: 序列化后符合 OpenAI 工具定义格式
// ============================================================================
#[test]
fn test_tool_json_format() {
    let tool = json!({
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "Get current weather information",
            "parameters": {
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "City name"
                    }
                },
                "required": ["city"]
            }
        }
    });

    // 验证基本结构
    assert_eq!(tool["type"], "function");
    assert!(tool.get("function").is_some());

    // 验证 function 字段
    let function = &tool["function"];
    assert_eq!(function["name"], "get_weather");
    assert_eq!(function["description"], "Get current weather information");

    // 验证 parameters 结构
    let params = &function["parameters"];
    assert_eq!(params["type"], "object");
    assert!(params.get("properties").is_some());
    assert!(params["required"].is_array());

    // 验证可以序列化为字符串
    let tool_json = serde_json::to_string(&tool).unwrap();
    assert!(tool_json.contains("\"type\":\"function\""));
    assert!(tool_json.contains("\"name\":\"get_weather\""));
}

// ============================================================================
// Test: test_tools_array_format
// 输入: json!([tool1, tool2])
// 预期: 正确序列化为数组
// ============================================================================
#[test]
fn test_tools_array_format() {
    let tools = json!([
        {
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get weather",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "city": {"type": "string"}
                    },
                    "required": ["city"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "get_time",
                "description": "Get current time",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "timezone": {"type": "string"}
                    },
                    "required": ["timezone"]
                }
            }
        }
    ]);

    // 验证是数组
    assert!(tools.is_array());
    let tools_array = tools.as_array().unwrap();
    assert_eq!(tools_array.len(), 2);

    // 验证第一个工具
    assert_eq!(tools_array[0]["function"]["name"], "get_weather");

    // 验证第二个工具
    assert_eq!(tools_array[1]["function"]["name"], "get_time");

    // 验证可以序列化
    let tools_json = serde_json::to_string(&tools).unwrap();
    assert!(tools_json.starts_with("["));
    assert!(tools_json.ends_with("]"));
}

// ============================================================================
// Test: test_tool_choice_string
// 输入: tool_choice 字符串格式
// 预期: 正确处理 "auto", "none", "required"
// ============================================================================
#[test]
fn test_tool_choice_string() {
    let choices = vec!["auto", "none", "required"];

    for choice in choices {
        let tool_choice = json!(choice);

        // 验证是字符串
        assert!(tool_choice.is_string());
        assert_eq!(tool_choice.as_str().unwrap(), choice);

        // 验证可以序列化
        let json_str = serde_json::to_string(&tool_choice).unwrap();
        assert_eq!(json_str, format!("\"{}\"", choice));
    }
}

// ============================================================================
// Test: test_tool_choice_object
// 输入: tool_choice 对象格式
// 预期: 正确指定特定函数
// ============================================================================
#[test]
fn test_tool_choice_object() {
    let tool_choice = json!({
        "type": "function",
        "function": {
            "name": "get_weather"
        }
    });

    // 验证结构
    assert_eq!(tool_choice["type"], "function");
    assert_eq!(tool_choice["function"]["name"], "get_weather");

    // 验证可以序列化
    let json_str = serde_json::to_string(&tool_choice).unwrap();
    assert!(json_str.contains("\"type\":\"function\""));
    assert!(json_str.contains("\"name\":\"get_weather\""));
}

// ============================================================================
// Test: test_tool_without_description
// 输入: 没有描述的工具定义
// 预期: 仍然有效
// ============================================================================
#[test]
fn test_tool_without_description() {
    let tool = json!({
        "type": "function",
        "function": {
            "name": "simple_tool",
            "parameters": {
                "type": "object",
                "properties": {}
            }
        }
    });

    // 验证基本字段存在
    assert_eq!(tool["type"], "function");
    assert_eq!(tool["function"]["name"], "simple_tool");

    // description 字段可选
    assert!(
        tool["function"].get("description").is_none() || tool["function"]["description"].is_null()
    );
}

// ============================================================================
// Test: test_tool_with_complex_parameters
// 输入: 复杂参数类型的工具
// 预期: 正确处理嵌套结构
// ============================================================================
#[test]
fn test_tool_with_complex_parameters() {
    let tool = json!({
        "type": "function",
        "function": {
            "name": "search",
            "description": "Search for items",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "filters": {
                        "type": "object",
                        "properties": {
                            "category": {"type": "string"},
                            "price_range": {
                                "type": "object",
                                "properties": {
                                    "min": {"type": "number"},
                                    "max": {"type": "number"}
                                }
                            }
                        }
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max results"
                    }
                },
                "required": ["query"]
            }
        }
    });

    // 验证嵌套结构
    let params = &tool["function"]["parameters"];
    assert!(params["properties"]["filters"].is_object());

    let filters = &params["properties"]["filters"];
    assert!(filters["properties"]["price_range"].is_object());

    // 验证必需参数
    let required = params["required"].as_array().unwrap();
    assert_eq!(required.len(), 1);
    assert_eq!(required[0], "query");
}

// ============================================================================
// Test: test_tool_parameter_types
// 输入: 不同参数类型
// 预期: 支持所有 JSON Schema 类型
// ============================================================================
#[test]
fn test_tool_parameter_types() {
    let tool = json!({
        "type": "function",
        "function": {
            "name": "multi_type_params",
            "parameters": {
                "type": "object",
                "properties": {
                    "string_param": {"type": "string"},
                    "number_param": {"type": "number"},
                    "integer_param": {"type": "integer"},
                    "boolean_param": {"type": "boolean"},
                    "array_param": {
                        "type": "array",
                        "items": {"type": "string"}
                    },
                    "object_param": {
                        "type": "object",
                        "properties": {}
                    }
                }
            }
        }
    });

    let props = &tool["function"]["parameters"]["properties"];

    // 验证不同类型
    assert_eq!(props["string_param"]["type"], "string");
    assert_eq!(props["number_param"]["type"], "number");
    assert_eq!(props["integer_param"]["type"], "integer");
    assert_eq!(props["boolean_param"]["type"], "boolean");
    assert_eq!(props["array_param"]["type"], "array");
    assert_eq!(props["object_param"]["type"], "object");

    // 验证数组项类型
    assert_eq!(props["array_param"]["items"]["type"], "string");
}

// ============================================================================
// Test: test_tool_with_enum
// 输入: 包含枚举的参数
// 预期: 正确定义枚举值
// ============================================================================
#[test]
fn test_tool_with_enum() {
    let tool = json!({
        "type": "function",
        "function": {
            "name": "set_mode",
            "parameters": {
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["fast", "normal", "slow"]
                    }
                },
                "required": ["mode"]
            }
        }
    });

    let mode_param = &tool["function"]["parameters"]["properties"]["mode"];
    assert_eq!(mode_param["type"], "string");

    let enum_values = mode_param["enum"].as_array().unwrap();
    assert_eq!(enum_values.len(), 3);
    assert_eq!(enum_values[0], "fast");
    assert_eq!(enum_values[1], "normal");
    assert_eq!(enum_values[2], "slow");
}

// ============================================================================
// Test: test_multiple_tools_with_same_request
// 输入: 多个工具同时定义
// 预期: 可以正确处理多个工具
// ============================================================================
#[test]
fn test_multiple_tools_with_same_request() {
    let tools = json!([
        {
            "type": "function",
            "function": {
                "name": "tool_a",
                "description": "Tool A",
                "parameters": {"type": "object", "properties": {}}
            }
        },
        {
            "type": "function",
            "function": {
                "name": "tool_b",
                "description": "Tool B",
                "parameters": {"type": "object", "properties": {}}
            }
        },
        {
            "type": "function",
            "function": {
                "name": "tool_c",
                "description": "Tool C",
                "parameters": {"type": "object", "properties": {}}
            }
        }
    ]);

    assert_eq!(tools.as_array().unwrap().len(), 3);

    // 验证每个工具都有唯一的名称
    let names: Vec<&str> = tools
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["function"]["name"].as_str().unwrap())
        .collect();

    assert_eq!(names, vec!["tool_a", "tool_b", "tool_c"]);
}

// ============================================================================
// Test: test_tool_serialization_roundtrip
// 输入: 工具定义序列化后反序列化
// 预期: 数据保持一致
// ============================================================================
#[test]
fn test_tool_serialization_roundtrip() {
    let original = json!({
        "type": "function",
        "function": {
            "name": "test_function",
            "description": "A test function",
            "parameters": {
                "type": "object",
                "properties": {
                    "arg1": {"type": "string"}
                },
                "required": ["arg1"]
            }
        }
    });

    // 序列化
    let json_str = serde_json::to_string(&original).unwrap();

    // 反序列化
    let deserialized: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // 验证数据一致
    assert_eq!(original, deserialized);
}
