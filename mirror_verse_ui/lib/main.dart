import 'dart:io';

import 'package:flutter/material.dart';

import 'functions.dart';
import 'new_generation.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Mirror Verse',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      debugShowCheckedModeBanner: false,
      home: const MyHomePage(),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key});

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  List<File> fileList = [];
  File? selectedFile;

  void loadFiles() {
    fileList = [];
    final Directory directory = Directory('../assets/');
    final List<FileSystemEntity> files = directory.listSync();
    for (final FileSystemEntity file in files) {
      if (file is File) {
        fileList.add(file);
      }
    }
  }

  @override
  void initState() {
    loadFiles();
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: const Text('Mirror Verse'),
        centerTitle: true,
        actions: [
          Image.asset('assets/logo.png'),
        ],
      ),
      body: Center(
        child: Column(
          children: [
            SizedBox(
              height: MediaQuery.of(context).size.height * 0.5,
              child: SingleChildScrollView(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    for (var i = 0; i < fileList.length; i++)
                      InkWell(
                        child: Container(
                          width: MediaQuery.of(context).size.width,
                          padding: const EdgeInsets.all(12),
                          color:
                              selectedFile == fileList[i] ? Colors.grey : null,
                          child: Text(
                            fileList[i]
                                .path
                                .replaceAll("../assets/", "")
                                .replaceAll(".json", ""),
                          ),
                        ),
                        onTap: () {
                          setState(() {
                            selectedFile = fileList[i];
                          });
                        },
                      ),
                  ],
                ),
              ),
            ),
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                ElevatedButton(
                  onPressed: () {
                    deleteFile(selectedFile!);
                    setState(() {
                      loadFiles();
                    });
                  },
                  child: const Text('Supprimer'),
                ),
                const SizedBox(width: 10),
                ElevatedButton(
                  onPressed: () {
                    Navigator.push(
                      context,
                      MaterialPageRoute(
                        builder: (context) => const NewGeneration(),
                      ),
                    ).then((value) {
                      setState(() {
                        loadFiles();
                      });
                    });
                  },
                  child: const Text('Nouveau'),
                ),
              ],
            ),
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () {
          if (selectedFile == null || selectedFile!.path.isEmpty) {
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(
                content: Text('Veuillez sélectionner un fichier'),
              ),
            );
            return;
          }
          runGeneration(selectedFile!);
        },
        tooltip: 'Run',
        child: const Icon(Icons.play_arrow),
      ), // This trailing comma makes auto-formatting nicer for build methods.
    );
  }
}
