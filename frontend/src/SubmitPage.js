import React from 'react';
import { Input, Button, PageHeader } from 'antd';
import './stylesheets/SubmitPage.css'

const axios = require('axios');
const { TextArea } = Input;


class SubmitPage extends React.Component {
	constructor(props) {
		super(props);
		this.state = {
			form: {
				source_code: "",
				language: "",
				user_id: 0,
				contest_id: 0,
				problem_id: 0
			},
			dataType: {
				source_code: "string",
				language: "string",
				user_id: "int", 
				contest_id: "int", 
				problem_id: "int"
			}
		};
		this.handleChangeState = this.handleChangeState.bind(this);
		this.handleSubmit = this.handleSubmit.bind(this);
	}

	handleChangeState(e) {
		var type = e.target.dataset.type;
		var value = e.target.value;
		var oldForm = this.state.form;
		if (this.state.dataType[type] == "int") 
			value = parseInt(value);
		oldForm[type] = value;
		this.setState({
			form: oldForm
		});
	}

	handleSubmit(e) {
		var data = this.state.form;

		axios.post("/jobs", data).then(res => {
			if (res && res.status === 200) {
				console.log(res.data);
				this.props.handlePageChange("status");
				this.props.handleJobIdChange(res.data.id);
			} else {
				console.log("Status " + res.status);
			}
		})
		.catch(err => {
			console.log(err);
		});
	}

	render() {
		return <div className="SubmitPage">
			<div id='head'>
				<PageHeader title='提交代码' /> 
			</div>
			<div id='body'>
				<div className='item'>
					<span>用户编号</span>
					<Input placeholder="0" data-type="user_id" onChange={this.handleChangeState}/>
				</div>
				<div className='item'>
					<span>比赛编号</span>
					<Input placeholder='0' data-type="contest_id" onChange={this.handleChangeState}/>
				</div>
				<div className='item'>
					<span>题目编号</span>
					<Input placeholder='0' data-type="problem_id" onChange={this.handleChangeState}/>
				</div>
				<div className='item'>
					<span>编程语言</span>
					<Input placeholder='Rust' data-type="language" onChange={this.handleChangeState}/>
				</div>
				<div className='item'>
					<p>提交代码</p>
					<TextArea autoSize={{ minRows: 5 }} placeholder="Your code here ... " data-type="source_code" onChange={this.handleChangeState}/>
				</div>
			</div>
			<Button onClick={this.handleSubmit}>提交</Button>
		</div>
	}
}

export default SubmitPage;