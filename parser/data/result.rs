pub fn truc() {
    let result = MapSpec {
        map: MapBlock {
            directives: [
                Srid(Srid { value: 3857 }),
                Extent(Extent {
                    minx: Integer(11111),
                    miny: Integer(333333),
                    maxx: Float(22222.2),
                    maxy: Integer(444444),
                }),
                Data(Data {
                    ident: "blue",
                    constructor: Val(Fn(FunctionCall {
                        name: "rgb",
                        args: [
                            Lit(Number(Integer(0))),
                            Lit(Number(Integer(0))),
                            Lit(Number(Integer(255))),
                        ],
                    })),
                }),
                Data(Data {
                    ident: "prop1",
                    constructor: Select {
                        selector: "col1",
                        datatype: String,
                    },
                }),
                Data(Data {
                    ident: "prop2",
                    constructor: Select {
                        selector: "col2",
                        datatype: Number,
                    },
                }),
                Data(Data {
                    ident: "red",
                    constructor: Val(Fn(FunctionCall {
                        name: "rgb",
                        args: [
                            Lit(Number(Integer(255))),
                            Lit(Number(Integer(30))),
                            Lit(Number(Integer(0))),
                        ],
                    })),
                }),
                Data(Data {
                    ident: "truc",
                    constructor: Val(Lit(String("a string"))),
                }),
            ],
        },
        layers: [LayerBlock {
            directives: [
                Source(Source {
                    driver: Postgis,
                    path: "user:pwd@localhost/schema_name/table_name",
                    srid: Some(Integer(31370)),
                }),
                Sym(Sym {
                    predicate: And {
                        left: Or {
                            left: Pred(Equal(ValuePair(
                                Fn(FunctionCall {
                                    name: "lowercase",
                                    args: [Data(Data {
                                        ident: "prop1",
                                        constructor: Select {
                                            selector: "col1",
                                            datatype: String,
                                        },
                                    })],
                                }),
                                Data(Data {
                                    ident: "truc",
                                    constructor: Val(Lit(String("a string"))),
                                }),
                            ))),
                            right: Or {
                                left: Or {
                                    left: Pred(Equal(ValuePair(
                                        Data(Data {
                                            ident: "prop1",
                                            constructor: Select {
                                                selector: "col1",
                                                datatype: String,
                                            },
                                        }),
                                        Lit(String("bench")),
                                    ))),
                                    right: Pred(Equal(ValuePair(
                                        Data(Data {
                                            ident: "prop1",
                                            constructor: Select {
                                                selector: "col1",
                                                datatype: String,
                                            },
                                        }),
                                        Lit(String("chair")),
                                    ))),
                                },
                                right: Pred(Equal(ValuePair(
                                    Data(Data {
                                        ident: "prop1",
                                        constructor: Select {
                                            selector: "col1",
                                            datatype: String,
                                        },
                                    }),
                                    Lit(String("something else")),
                                ))),
                            },
                        },
                        right: Pred(GreaterThanOrEqual(ValuePair(
                            Data(Data {
                                ident: "prop2",
                                constructor: Select {
                                    selector: "col2",
                                    datatype: Number,
                                },
                            }),
                            Lit(Number(Integer(12))),
                        ))),
                    },
                    consequent: [
                        Circle(Circle {
                            radius: Lit(Number(Integer(6))),
                        }),
                        Fill(Fill {
                            color: Data(Data {
                                ident: "red",
                                constructor: Val(Fn(FunctionCall {
                                    name: "rgb",
                                    args: [
                                        Lit(Number(Integer(255))),
                                        Lit(Number(Integer(30))),
                                        Lit(Number(Integer(0))),
                                    ],
                                })),
                            }),
                        }),
                    ],
                }),
                Sym(Sym {
                    predicate: Pred(Equal(ValuePair(
                        Data(Data {
                            ident: "prop1",
                            constructor: Select {
                                selector: "col1",
                                datatype: String,
                            },
                        }),
                        Lit(String("bin")),
                    ))),
                    consequent: [
                        Square(Square {
                            size: Lit(Number(Integer(8))),
                        }),
                        Fill(Fill {
                            color: Fn(FunctionCall {
                                name: "rgb",
                                args: [
                                    Lit(Number(Integer(12))),
                                    Lit(Number(Integer(34))),
                                    Lit(Number(Integer(56))),
                                ],
                            }),
                        }),
                    ],
                }),
            ],
        }],
    };
}
