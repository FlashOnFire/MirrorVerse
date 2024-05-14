import 'dart:io';

import 'package:flutter/material.dart';

import 'functions.dart';

class NewGeneration extends StatefulWidget {
  const NewGeneration({super.key});

  @override
  State<NewGeneration> createState() => _NewGenerationState();
}

class _NewGenerationState extends State<NewGeneration> {
  String? name;
  int mirrorCount = 12;
  int dimCount = 3;
  int raysCount = 4;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: const Text('Mirror Verse - Génération de mirroir'),
        centerTitle: true,
        actions: [
          Image.asset('assets/logo.png'),
        ],
      ),
      body: Center(
        child: SizedBox(
          width: MediaQuery.of(context).size.width * 0.5,
          child: Column(
            children: [
              Padding(
                padding: const EdgeInsets.all(8.0),
                child: TextField(
                  onChanged: (value) {
                    setState(() {
                      name = value;
                    });
                  },
                  decoration: const InputDecoration(
                    labelText: 'Nom de la simulation',
                  ),
                ),
              ),
              Column(
                children: [
                  ListTile(
                    title: const Text("Nombre de dimensions"),
                    trailing: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        IconButton(
                          onPressed: () {
                            setState(() {
                              dimCount = dimCount - 1;
                            });
                          },
                          icon: const Icon(Icons.remove),
                        ),
                        SizedBox(
                          width: 40,
                          child: TextField(
                            onChanged: (value) {
                              dimCount = int.parse(value);
                            },
                            keyboardType: TextInputType.number,
                            controller: TextEditingController(
                                text: dimCount.toString()),
                          ),
                        ),
                        IconButton(
                          onPressed: () {
                            setState(() {
                              dimCount = dimCount + 1;
                            });
                          },
                          icon: const Icon(Icons.add),
                        ),
                      ],
                    ),
                  ),
                  if (dimCount > 3 || dimCount < 2)
                    const Text(
                      "Attention, les dimensions supérieures à 3 ou inférieures à 2 ne pourront pas être visualisées",
                      style: TextStyle(
                          color: Colors.red, fontWeight: FontWeight.bold),
                    ),
                  ListTile(
                    title: const Text("Nombre de mirroir"),
                    trailing: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        IconButton(
                          onPressed: () {
                            setState(() {
                              mirrorCount = mirrorCount - 1;
                            });
                          },
                          icon: const Icon(Icons.remove),
                        ),
                        SizedBox(
                          width: 40,
                          child: TextField(
                            onChanged: (value) {
                              mirrorCount = int.parse(value);
                            },
                            keyboardType: TextInputType.number,
                            controller: TextEditingController(
                                text: mirrorCount.toString()),
                          ),
                        ),
                        IconButton(
                          onPressed: () {
                            setState(() {
                              mirrorCount = mirrorCount + 1;
                            });
                          },
                          icon: const Icon(Icons.add),
                        ),
                      ],
                    ),
                  ),
                  ListTile(
                    title: const Text("Nombre de rayons"),
                    trailing: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        IconButton(
                          onPressed: () {
                            setState(() {
                              raysCount = raysCount - 1;
                            });
                          },
                          icon: const Icon(Icons.remove),
                        ),
                        SizedBox(
                          width: 40,
                          child: TextField(
                            onChanged: (value) {
                              raysCount = int.parse(value);
                            },
                            keyboardType: TextInputType.number,
                            controller: TextEditingController(
                                text: raysCount.toString()),
                          ),
                        ),
                        IconButton(
                          onPressed: () {
                            setState(() {
                              raysCount = raysCount + 1;
                            });
                          },
                          icon: const Icon(Icons.add),
                        ),
                      ],
                    ),
                  )
                ],
              )
            ],
          ),
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () {
          if (name != null && name!.isNotEmpty) {
            generateMirrorSet(name!, dimCount, mirrorCount, raysCount)
                .then((value) => showDialog(
                    context: context,
                    builder: (context) {
                      return AlertDialog(
                        title: const Text('Génération terminée'),
                        content: const Text(
                            'La génération de la simulation est terminée, voulez vous la lancer ?'),
                        actions: [
                          TextButton(
                            onPressed: () {
                              Navigator.of(context).pop();
                              Navigator.of(context).pop();
                              runGeneration(File('../assets/$name.json'));
                            },
                            child: const Text('OK'),
                          ),
                          TextButton(
                            onPressed: () {
                              Navigator.of(context).pop();
                            },
                            child: const Text('Annuler'),
                          ),
                        ],
                      );
                    }));
          } else {
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(
                content: Text('Veuillez entrer un nom de simulation'),
              ),
            );
          }
        },
        tooltip: 'Run',
        child: const Icon(Icons.play_arrow),
      ),
    );
  }
}
