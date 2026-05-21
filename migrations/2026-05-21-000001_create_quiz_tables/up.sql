CREATE TABLE questions (
    id SERIAL PRIMARY KEY,
    topic TEXT NOT NULL,
    difficulty TEXT NOT NULL DEFAULT 'beginner',
    question_text TEXT NOT NULL,
    options JSONB NOT NULL,
    correct_answer TEXT NOT NULL
);

CREATE TABLE quiz_attempts (
    id SERIAL PRIMARY KEY,
    player_name TEXT NOT NULL,
    player_email TEXT NOT NULL,
    topic TEXT,
    difficulty TEXT NOT NULL DEFAULT 'beginner',
    num_questions INTEGER NOT NULL DEFAULT 10,
    score INTEGER NOT NULL DEFAULT 0,
    total_questions INTEGER NOT NULL DEFAULT 0,
    answers_json JSONB,
    played_at TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO questions (topic, difficulty, question_text, options, correct_answer) VALUES
-- Technology (10 questions)
('Technology', 'beginner', 'What does CPU stand for?', '["Central Processing Unit", "Computer Personal Unit", "Central Program Utility", "Core Processing Unit"]', 'Central Processing Unit'),
('Technology', 'beginner', 'What is the most common operating system for personal computers?', '["Microsoft Windows", "Linux", "macOS", "Android"]', 'Microsoft Windows'),
('Technology', 'beginner', 'Which company developed the iPhone?', '["Apple", "Samsung", "Google", "Microsoft"]', 'Apple'),
('Technology', 'intermediate', 'What does "HTTP" stand for?', '["HyperText Transfer Protocol", "High Transfer Text Protocol", "HyperText Transmission Process", "Highway Text Transfer Protocol"]', 'HyperText Transfer Protocol'),
('Technology', 'intermediate', 'Which programming language is primarily used for Android app development?', '["Kotlin", "Swift", "Python", "JavaScript"]', 'Kotlin'),
('Technology', 'intermediate', 'What is the purpose of a firewall?', '["Network security", "Increase internet speed", "Store passwords", "Manage emails"]', 'Network security'),
('Technology', 'expert', 'What is the difference between TCP and UDP?', '["TCP is connection-oriented, UDP is connectionless", "UDP is faster but unreliable, TCP is reliable", "Both A and B", "TCP uses ports, UDP does not"]', 'Both A and B'),
('Technology', 'expert', 'What does SQL injection exploit?', '["Unsanitized user input in database queries", "Weak passwords", "Outdated software", "Network vulnerabilities"]', 'Unsanitized user input in database queries'),
('Technology', 'expert', 'Which protocol is used for secure web communication?', '["HTTPS/TLS", "HTTP", "FTP", "SMTP"]', 'HTTPS/TLS'),
('Technology', 'intermediate', 'What is the role of a DNS server?', '["Translate domain names to IP addresses", "Store website content", "Manage email servers", "Provide internet connectivity"]', 'Translate domain names to IP addresses'),

-- Science (10 questions)
('Science', 'beginner', 'What is the chemical symbol for water?', '["H2O", "CO2", "NaCl", "O2"]', 'H2O'),
('Science', 'beginner', 'Which planet is known as the Red Planet?', '["Mars", "Venus", "Jupiter", "Saturn"]', 'Mars'),
('Science', 'beginner', 'What gas do plants absorb from the atmosphere?', '["Carbon Dioxide", "Oxygen", "Nitrogen", "Hydrogen"]', 'Carbon Dioxide'),
('Science', 'intermediate', 'What is the powerhouse of the cell?', '["Mitochondria", "Nucleus", "Ribosome", "Golgi apparatus"]', 'Mitochondria'),
('Science', 'intermediate', 'What is the speed of light in vacuum?', '["~300,000 km/s", "~150,000 km/s", "~500,000 km/s", "~100,000 km/s"]', '~300,000 km/s'),
('Science', 'intermediate', 'Which element has the atomic number 1?', '["Hydrogen", "Helium", "Lithium", "Oxygen"]', 'Hydrogen'),
('Science', 'expert', 'What is the Heisenberg Uncertainty Principle?', '["Cannot know both position and momentum precisely", "Energy cannot be created or destroyed", "Entropy always increases", "Light is both wave and particle"]', 'Cannot know both position and momentum precisely'),
('Science', 'expert', 'What is CRISPR used for?', '["Gene editing", "Cancer treatment", "Creating vaccines", "Weather prediction"]', 'Gene editing'),
('Science', 'expert', 'What is dark matter?', '["Unknown form of matter that doesn''t emit light", "Black holes", "Dead stars", "Interstellar gas"]', 'Unknown form of matter that doesn''t emit light'),
('Science', 'intermediate', 'What is the pH of pure water?', '["7", "1", "14", "0"]', '7'),

-- History (8 questions)
('History', 'beginner', 'Who discovered America?', '["Christopher Columbus", "Vasco da Gama", "Ferdinand Magellan", "Marco Polo"]', 'Christopher Columbus'),
('History', 'beginner', 'What year did World War II end?', '["1945", "1944", "1946", "1943"]', '1945'),
('History', 'beginner', 'Who was the first President of the United States?', '["George Washington", "Thomas Jefferson", "Abraham Lincoln", "John Adams"]', 'George Washington'),
('History', 'intermediate', 'What was the Renaissance?', '["Cultural rebirth in Europe", "A war", "A plague", "A religious movement"]', 'Cultural rebirth in Europe'),
('History', 'intermediate', 'Which empire built the Colosseum?', '["Roman Empire", "Greek Empire", "Egyptian Empire", "Persian Empire"]', 'Roman Empire'),
('History', 'intermediate', 'What was the Industrial Revolution?', '["Shift to manufacturing and industry", "Political revolution", "Agricultural change", "Digital transformation"]', 'Shift to manufacturing and industry'),
('History', 'expert', 'What event started World War I?', '["Assassination of Archduke Franz Ferdinand", "Invasion of Poland", "Battle of Waterloo", "Fall of Berlin Wall"]', 'Assassination of Archduke Franz Ferdinand'),
('History', 'expert', 'What was the Cold War?', '["Geopolitical tension between US and USSR", "A military war", "A trade war", "A religious conflict"]', 'Geopolitical tension between US and USSR'),

-- Geography (8 questions)
('Geography', 'beginner', 'What is the largest continent?', '["Asia", "Africa", "North America", "Europe"]', 'Asia'),
('Geography', 'beginner', 'Which is the longest river in the world?', '["Nile", "Amazon", "Mississippi", "Yangtze"]', 'Nile'),
('Geography', 'beginner', 'What is the capital of Japan?', '["Tokyo", "Kyoto", "Osaka", "Seoul"]', 'Tokyo'),
('Geography', 'intermediate', 'Which country has the largest population?', '["India", "China", "USA", "Indonesia"]', 'India'),
('Geography', 'intermediate', 'What is the smallest country in the world?', '["Vatican City", "Monaco", "San Marino", "Liechtenstein"]', 'Vatican City'),
('Geography', 'intermediate', 'Which desert is the largest hot desert?', '["Sahara", "Gobi", "Kalahari", "Arabian"]', 'Sahara'),
('Geography', 'expert', 'What is the capital of Mongolia?', '["Ulaanbaatar", "Bishkek", "Astana", "Tashkent"]', 'Ulaanbaatar'),
('Geography', 'expert', 'Which country is both in Europe and Asia?', '["Russia", "Turkey", "Both A and B", "Egypt"]', 'Both A and B'),

-- Programming (10 questions)
('Programming', 'beginner', 'What does HTML stand for?', '["HyperText Markup Language", "High Text Machine Language", "HyperText Modern Language", "Home Tool Markup Language"]', 'HyperText Markup Language'),
('Programming', 'beginner', 'Which language is mainly used for web styling?', '["CSS", "HTML", "JavaScript", "Python"]', 'CSS'),
('Programming', 'beginner', 'What is a variable?', '["A storage location with a name", "A constant value", "A function", "A loop"]', 'A storage location with a name'),
('Programming', 'intermediate', 'What is the time complexity of binary search?', '["O(log n)", "O(n)", "O(n log n)", "O(1)"]', 'O(log n)'),
('Programming', 'intermediate', 'What is an API?', '["Interface for software applications", "A programming language", "A database", "A web server"]', 'Interface for software applications'),
('Programming', 'intermediate', 'What does OOP stand for?', '["Object-Oriented Programming", "Online Operating Process", "Object Order Protocol", "Official Operation Program"]', 'Object-Oriented Programming'),
('Programming', 'expert', 'What is a deadlock in concurrent programming?', '["Two processes waiting for each other''s resources", "A crashed program", "An infinite loop", "Memory overflow"]', 'Two processes waiting for each other''s resources'),
('Programming', 'expert', 'What is Rust''s ownership model?', '["Each value has one owner", "Multiple owners allowed", "No memory management", "Garbage collected"]', 'Each value has one owner'),
('Programming', 'expert', 'What is the difference between mutex and semaphore?', '["Mutex allows one thread, semaphore allows N threads", "Mutex is faster", "Semaphore is only for processes", "No difference"]', 'Mutex allows one thread, semaphore allows N threads'),
('Programming', 'intermediate', 'What is Git?', '["Version control system", "Programming language", "Database", "Web framework"]', 'Version control system'),

-- General Knowledge (8 questions)
('General Knowledge', 'beginner', 'How many days are in a leap year?', '["366", "365", "364", "367"]', '366'),
('General Knowledge', 'beginner', 'What is the largest mammal?', '["Blue Whale", "Elephant", "Giraffe", "Hippopotamus"]', 'Blue Whale'),
('General Knowledge', 'beginner', 'What color are bananas when ripe?', '["Yellow", "Green", "Red", "Blue"]', 'Yellow'),
('General Knowledge', 'intermediate', 'Which language has the most native speakers?', '["Mandarin Chinese", "English", "Spanish", "Hindi"]', 'Mandarin Chinese'),
('General Knowledge', 'intermediate', 'What is the boiling point of water in Celsius?', '["100°C", "0°C", "50°C", "212°C"]', '100°C'),
('General Knowledge', 'intermediate', 'Who painted the Mona Lisa?', '["Leonardo da Vinci", "Michelangelo", "Raphael", "Van Gogh"]', 'Leonardo da Vinci'),
('General Knowledge', 'expert', 'What is the tallest mountain in the world?', '["Mount Everest", "K2", "Kangchenjunga", "Lhotse"]', 'Mount Everest'),
('General Knowledge', 'expert', 'What is the currency of Japan?', '["Japanese Yen", "Chinese Yuan", "Korean Won", "Thai Baht"]', 'Japanese Yen');
