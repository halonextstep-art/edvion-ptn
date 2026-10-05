import { useState } from 'react';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Textarea } from '../ui/textarea';
import { Card } from '../ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { 
  Save, 
  Eye, 
  FileText,
  Code,
  Image as ImageIcon,
  Plus,
  X,
  Sparkles
} from 'lucide-react';
import { Badge } from '../ui/badge';
import { useApp } from '../../lib/AppContext';
import { Question } from '../../lib/mockData';
import { toast } from 'sonner';

interface QuestionEditorProps {
  onClose: () => void;
  editQuestion?: Question;
}

export default function QuestionEditor({ onClose, editQuestion }: QuestionEditorProps) {
  const { addQuestion, updateQuestion } = useApp();
  const isEditing = !!editQuestion;

  const [questionType, setQuestionType] = useState<'multiple_choice' | 'complex_multiple' | 'short_answer'>(
    editQuestion?.type || 'multiple_choice'
  );
  const [subject, setSubject] = useState(editQuestion?.subject || 'Matematika');
  const [topic, setTopic] = useState(editQuestion?.topic || '');
  const [subtopic, setSubtopic] = useState(editQuestion?.subtopic || '');
  const [difficulty, setDifficulty] = useState<'easy' | 'medium' | 'hard'>(editQuestion?.difficulty || 'medium');
  const [bloomLevel, setBloomLevel] = useState(editQuestion?.bloomLevel || 'C3 - Aplikasi');
  const [hasStimulus, setHasStimulus] = useState(!!editQuestion?.stimulus);
  const [stimulus, setStimulus] = useState(editQuestion?.stimulus || '');
  const [question, setQuestion] = useState(editQuestion?.question || '');
  const [options, setOptions] = useState<string[]>(
    editQuestion?.type === 'multiple_choice' && Array.isArray(editQuestion.options) 
      ? editQuestion.options 
      : ['', '', '', '', '']
  );
  const [correctAnswer, setCorrectAnswer] = useState(editQuestion?.correctAnswer?.toString() || '');
  const [explanation, setExplanation] = useState(editQuestion?.explanation || '');
  const [tags, setTags] = useState<string[]>(editQuestion?.tags || []);
  const [newTag, setNewTag] = useState('');

  const handleSave = () => {
    // Validation
    if (!question.trim()) {
      toast.error('Pertanyaan tidak boleh kosong!');
      return;
    }

    if (questionType === 'multiple_choice' && options.some(opt => !opt.trim())) {
      toast.error('Semua opsi jawaban harus diisi!');
      return;
    }

    if (!correctAnswer) {
      toast.error('Jawaban yang benar harus ditentukan!');
      return;
    }

    const questionData: Question = {
      id: editQuestion?.id || `Q${String(Date.now()).slice(-3)}`,
      code: editQuestion?.code || `${subject.substring(0, 3).toUpperCase()}-${topic.substring(0, 3).toUpperCase()}-${String(Date.now()).slice(-3)}`,
      type: questionType,
      subject,
      topic: topic || 'General',
      subtopic: subtopic || 'General',
      difficulty,
      bloomLevel,
      stimulus: hasStimulus ? stimulus : undefined,
      question,
      options: questionType === 'multiple_choice' ? options : undefined,
      correctAnswer,
      explanation,
      tags,
      createdBy: 'Admin Pusat',
      createdAt: editQuestion?.createdAt || new Date().toISOString().split('T')[0],
      usageCount: editQuestion?.usageCount || 0,
      averageScore: editQuestion?.averageScore || 0,
      timeLimit: 120
    };

    if (isEditing) {
      updateQuestion(editQuestion.id, questionData);
      toast.success('✅ Soal berhasil diperbarui!');
    } else {
      addQuestion(questionData);
      toast.success('✅ Soal baru berhasil ditambahkan!');
    }

    onClose();
  };

  const handleAddTag = () => {
    if (newTag.trim() && !tags.includes(newTag.trim())) {
      setTags([...tags, newTag.trim()]);
      setNewTag('');
    }
  };

  const handleRemoveTag = (tagToRemove: string) => {
    setTags(tags.filter(tag => tag !== tagToRemove));
  };

  return (
    <div className="space-y-6">
      {/* Step Indicator */}
      <div className="flex items-center justify-between">
        {['Jenis Soal', 'Konten', 'Jawaban', 'Preview'].map((step, idx) => (
          <div key={step} className="flex items-center">
            <div className={`flex items-center gap-2 ${idx === 0 ? 'text-indigo-600' : 'text-muted-foreground'}`}>
              <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm ${
                idx === 0 ? 'bg-indigo-600 text-white' : 'bg-slate-200'
              }`}>
                {idx + 1}
              </div>
              <span className="text-sm hidden sm:inline">{step}</span>
            </div>
            {idx < 3 && <div className="w-12 h-px bg-slate-200 mx-2"></div>}
          </div>
        ))}
      </div>

      <Tabs defaultValue="editor" className="space-y-4">
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="editor" className="gap-2">
            <FileText className="w-4 h-4" />
            Editor
          </TabsTrigger>
          <TabsTrigger value="preview" className="gap-2">
            <Eye className="w-4 h-4" />
            Preview
          </TabsTrigger>
        </TabsList>

        <TabsContent value="editor" className="space-y-6">
          {/* Question Type Selection */}
          <Card className="p-6 border-2 border-indigo-100 bg-indigo-50/30">
            <Label className="mb-3 block">Pilih Jenis Soal</Label>
            <div className="grid sm:grid-cols-3 gap-3">
              {[
                { value: 'multiple_choice' as const, label: 'Pilihan Ganda', icon: '📝' },
                { value: 'complex_multiple' as const, label: 'Pilihan Majemuk', icon: '📊' },
                { value: 'short_answer' as const, label: 'Isian Singkat', icon: '✍️' }
              ].map((type) => (
                <button
                  key={type.value}
                  onClick={() => setQuestionType(type.value)}
                  className={`p-4 rounded-lg border-2 transition-all ${
                    questionType === type.value
                      ? 'border-indigo-500 bg-indigo-50'
                      : 'border-slate-200 bg-white hover:border-slate-300'
                  }`}
                >
                  <div className="text-2xl mb-2">{type.icon}</div>
                  <div className="text-sm">{type.label}</div>
                </button>
              ))}
            </div>
          </Card>

          {/* Metadata */}
          <div className="grid sm:grid-cols-2 gap-4">
            <div>
              <Label>Mata Uji</Label>
              <select 
                className="w-full mt-1.5 px-3 py-2 border rounded-md"
                value={subject}
                onChange={(e) => setSubject(e.target.value)}
              >
                <option>Matematika</option>
                <option>Fisika</option>
                <option>Kimia</option>
                <option>Biologi</option>
                <option>Bahasa Indonesia</option>
                <option>Bahasa Inggris</option>
              </select>
            </div>
            
            <div>
              <Label>Tingkat Kesulitan</Label>
              <select 
                className="w-full mt-1.5 px-3 py-2 border rounded-md"
                value={difficulty}
                onChange={(e) => setDifficulty(e.target.value as 'easy' | 'medium' | 'hard')}
              >
                <option value="easy">Mudah</option>
                <option value="medium">Sedang</option>
                <option value="hard">Sulit</option>
              </select>
            </div>

            <div>
              <Label>Topik</Label>
              <Input 
                value={topic}
                onChange={(e) => setTopic(e.target.value)}
                placeholder="Contoh: Aljabar"
                className="mt-1.5"
              />
            </div>

            <div>
              <Label>Sub-Topik</Label>
              <Input 
                value={subtopic}
                onChange={(e) => setSubtopic(e.target.value)}
                placeholder="Contoh: Persamaan Kuadrat"
                className="mt-1.5"
              />
            </div>
          </div>

          {/* Stimulus Toggle */}
          <div className="flex items-center gap-3 p-4 bg-purple-50 rounded-lg border border-purple-200">
            <input 
              type="checkbox" 
              id="hasStimulus"
              checked={hasStimulus}
              onChange={(e) => setHasStimulus(e.target.checked)}
              className="w-4 h-4"
            />
            <label htmlFor="hasStimulus" className="flex-1 cursor-pointer">
              <div className="flex items-center gap-2">
                <Sparkles className="w-4 h-4 text-purple-600" />
                <span>Soal dengan Stimulus Panjang</span>
              </div>
              <p className="text-sm text-muted-foreground mt-1">
                Aktifkan jika soal memiliki bacaan/data pendukung (untuk penalaran & literasi)
              </p>
            </label>
          </div>

          {/* Stimulus Editor (if enabled) */}
          {hasStimulus && (
            <Card className="p-6 bg-purple-50/50 border-purple-200">
              <div className="flex items-center justify-between mb-4">
                <Label>Stimulus / Bacaan Pendukung</Label>
                <div className="flex gap-2">
                  <Button variant="outline" size="sm" className="gap-2">
                    <ImageIcon className="w-4 h-4" />
                    Upload Gambar
                  </Button>
                  <Button variant="outline" size="sm" className="gap-2">
                    <Code className="w-4 h-4" />
                    LaTeX
                  </Button>
                </div>
              </div>
              <Textarea 
                value={stimulus}
                onChange={(e) => setStimulus(e.target.value)}
                placeholder="Masukkan teks bacaan, data, atau konteks yang akan menjadi dasar untuk pertanyaan..."
                className="min-h-32 bg-white"
              />
              <p className="text-sm text-muted-foreground mt-2">
                💡 Tip: Stimulus yang baik memberikan informasi lengkap untuk menjawab sub-soal
              </p>
            </Card>
          )}

          {/* Question Content */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <Label>Pertanyaan</Label>
              <Button variant="ghost" size="sm" className="gap-1 text-indigo-600">
                <Code className="w-3.5 h-3.5" />
                Insert Equation
              </Button>
            </div>
            <Textarea 
              value={question}
              onChange={(e) => setQuestion(e.target.value)}
              placeholder="Tulis pertanyaan soal di sini... Contoh: Jika nilai x = 3, maka hasil dari 2x + 5 adalah..."
              className="min-h-24"
            />
          </div>

          {/* Options (for Multiple Choice) */}
          {questionType === 'multiple_choice' && (
            <div>
              <Label className="mb-3 block">Opsi Jawaban</Label>
              <div className="space-y-3">
                {options.map((option, idx) => (
                  <div key={idx} className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-full bg-slate-100 flex items-center justify-center text-sm">
                      {String.fromCharCode(65 + idx)}
                    </div>
                    <Input 
                      placeholder={`Opsi ${String.fromCharCode(65 + idx)}`}
                      value={option}
                      onChange={(e) => {
                        const newOptions = [...options];
                        newOptions[idx] = e.target.value;
                        setOptions(newOptions);
                      }}
                      className="flex-1"
                    />
                    <input 
                      type="radio" 
                      name="correct"
                      checked={correctAnswer === options[idx]}
                      onChange={() => setCorrectAnswer(options[idx])}
                      className="w-4 h-4"
                    />
                  </div>
                ))}
              </div>
              <p className="text-sm text-muted-foreground mt-2">
                ✅ Pilih radio button untuk menandai jawaban yang benar
              </p>
            </div>
          )}

          {/* Fill in the Blank */}
          {questionType === 'short_answer' && (
            <div>
              <Label className="mb-2 block">Jawaban yang Benar</Label>
              <Input 
                value={correctAnswer}
                onChange={(e) => setCorrectAnswer(e.target.value)}
                placeholder="Masukkan jawaban yang benar (misal: 36)"
                className="max-w-md"
              />
              <p className="text-sm text-muted-foreground mt-2">
                💡 Sistem akan otomatis memvalidasi format jawaban
              </p>
            </div>
          )}

          {/* Explanation */}
          <div>
            <Label className="mb-2 block">Pembahasan</Label>
            <Textarea 
              value={explanation}
              onChange={(e) => setExplanation(e.target.value)}
              placeholder="Tulis pembahasan dan penjelasan jawaban yang benar..."
              className="min-h-32"
            />
          </div>

          {/* Tags & Metadata */}
          <div>
            <Label className="mb-2 block">Tags & Kompetensi</Label>
            <div className="flex flex-wrap gap-2 mb-3">
              {tags.map((tag) => (
                <Badge key={tag} variant="outline" className="gap-1">
                  {tag}
                  <button onClick={() => handleRemoveTag(tag)} className="ml-1">
                    <X className="w-3 h-3" />
                  </button>
                </Badge>
              ))}
              <div className="flex gap-2">
                <Input 
                  value={newTag}
                  onChange={(e) => setNewTag(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && (e.preventDefault(), handleAddTag())}
                  placeholder="Tambah tag"
                  className="w-32 h-7 text-sm"
                />
                <Button onClick={handleAddTag} variant="ghost" size="sm" className="h-7 gap-1">
                  <Plus className="w-3 h-3" />
                  Add
                </Button>
              </div>
            </div>
          </div>
        </TabsContent>

        <TabsContent value="preview" className="space-y-4">
          <Card className="p-8 bg-white border-2">
            <div className="flex items-center justify-between mb-6">
              <Badge variant="outline">{subject}</Badge>
              <Badge className={
                difficulty === 'easy' ? 'bg-green-100 text-green-700' :
                difficulty === 'medium' ? 'bg-orange-100 text-orange-700' :
                'bg-red-100 text-red-700'
              }>
                {difficulty === 'easy' ? 'Mudah' : difficulty === 'medium' ? 'Sedang' : 'Sulit'}
              </Badge>
            </div>

            {hasStimulus && stimulus && (
              <div className="p-6 bg-slate-50 rounded-lg mb-6 border">
                <h4 className="mb-3 text-sm text-muted-foreground">Stimulus:</h4>
                <p className="text-sm leading-relaxed">{stimulus}</p>
              </div>
            )}

            <div className="mb-6">
              <h3 className="text-lg mb-4">
                {hasStimulus ? 'Pertanyaan 1:' : 'Pertanyaan:'}
              </h3>
              <p>{question || '[Belum ada pertanyaan]'}</p>
            </div>

            {questionType === 'multiple_choice' && (
              <div className="space-y-3">
                {options.map((opt, idx) => (
                  <div key={idx} className={`flex items-center gap-3 p-3 rounded-lg border ${
                    correctAnswer === opt ? 'bg-green-50 border-green-300' : 'hover:bg-slate-50'
                  }`}>
                    <div className="w-8 h-8 rounded-full border-2 flex items-center justify-center text-sm">
                      {String.fromCharCode(65 + idx)}
                    </div>
                    <span>{opt || `[Opsi ${String.fromCharCode(65 + idx)}]`}</span>
                  </div>
                ))}
              </div>
            )}

            {questionType === 'short_answer' && (
              <div className="p-4 bg-blue-50 rounded-lg border border-blue-200">
                <p className="text-sm text-muted-foreground mb-2">Jawaban yang benar:</p>
                <p className="font-mono">{correctAnswer || '[Belum diisi]'}</p>
              </div>
            )}
          </Card>
        </TabsContent>
      </Tabs>

      {/* Action Buttons */}
      <div className="flex items-center justify-between pt-4 border-t">
        <div className="flex gap-2">
          <Button variant="outline" onClick={onClose}>Batal</Button>
        </div>
        
        <div className="flex gap-2">
          <Button onClick={handleSave} className="gap-2 bg-gradient-to-r from-indigo-600 to-purple-600">
            <Sparkles className="w-4 h-4" />
            {isEditing ? 'Update Soal' : 'Publikasikan'}
          </Button>
        </div>
      </div>
    </div>
  );
}