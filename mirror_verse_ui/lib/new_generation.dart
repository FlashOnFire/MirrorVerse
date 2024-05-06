import 'package:flutter/material.dart';

import 'functions.dart';

class NewGeneration extends StatefulWidget {
  const NewGeneration({super.key});

  @override
  State<NewGeneration> createState() => _NewGenerationState();
}

class _NewGenerationState extends State<NewGeneration> {
  String? name;
  Map<MirrorType, int> mirrorCounts = {};

  @override
  void initState() {
    for (final MirrorType mirrorType in MirrorType.values) {
      mirrorCounts[mirrorType] = 0;
    }
    super.initState();
  }

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
              Expanded(
                child: ListView.builder(
                  itemCount: mirrorCounts.length,
                  itemBuilder: (context, index) {
                    final mirrorType = mirrorCounts.keys.elementAt(index);
                    final count = mirrorCounts[mirrorType];
                    return ListTile(
                      title: Text(mirrorType.name),
                      trailing: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          IconButton(
                            onPressed: () {
                              setState(() {
                                if (count! > 0) {
                                  mirrorCounts[mirrorType] = count - 1;
                                }
                              });
                            },
                            icon: const Icon(Icons.remove),
                          ),
                          Text(count.toString()),
                          IconButton(
                            onPressed: () {
                              setState(() {
                                mirrorCounts[mirrorType] = count! + 1;
                              });
                            },
                            icon: const Icon(Icons.add),
                          ),
                        ],
                      ),
                    );
                  },
                ),
              ),
            ],
          ),
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () {
          if (name != null && name!.isNotEmpty) {
            generateMirrorSet(name: name!, mirrorCounts: mirrorCounts);
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
